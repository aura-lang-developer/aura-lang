//! Cranelift Native Machine Code Backend for Aura Language.
//! Compiles Aura AST directly to native machine code (Mach-O on macOS, ELF on Linux)
//! and links against the standalone Aura runtime (libaura_runtime.a).

use crate::ast::*;
use cranelift_codegen::ir::types;
use cranelift_codegen::ir::{AbiParam, Block, InstBuilder, MemFlagsData, Signature, Value};
use cranelift_codegen::isa;
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{DataId, FuncId, Linkage, Module as CraneliftModule, default_libcall_names};
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[allow(dead_code)]
pub struct CraneliftBackend {
    module: ObjectModule,
    func_ids: HashMap<String, FuncId>,
    globals: HashMap<String, DataId>,

    // Runtime imported function IDs
    rt_start_id: FuncId,
    fiber_spawn_id: FuncId,
    fiber_yield_id: FuncId,
    chan_new_id: FuncId,
    chan_send_id: FuncId,
    chan_recv_id: FuncId,
    gc_alloc_id: FuncId,
    gc_safepoint_id: FuncId,
    gc_collect_id: FuncId,

    // I/O & Printing
    println_any_id: FuncId,
    println_str_id: FuncId,

    // Synchronization
    mutex_new_id: FuncId,
    mutex_lock_id: FuncId,
    mutex_unlock_id: FuncId,

    // OS & Utilities
    os_env_id: FuncId,
    os_hostname_id: FuncId,
    process_uptime_id: FuncId,
    time_now_id: FuncId,
    sleep_ms_id: FuncId,
    parse_float_id: FuncId,

    // Strings
    string_new_id: FuncId,
    string_concat_id: FuncId,
    string_eq_id: FuncId,
    string_to_lower_id: FuncId,
    string_contains_id: FuncId,
    string_len_id: FuncId,
    string_cstr_id: FuncId,

    // Records / Structs
    record_new_id: FuncId,
    record_set_id: FuncId,
    record_get_id: FuncId,
    record_to_json_id: FuncId,

    // Lists
    list_new_id: FuncId,
    list_push_id: FuncId,
    list_get_id: FuncId,
    list_set_id: FuncId,
    list_len_id: FuncId,
    list_cap_id: FuncId,
    list_slice_id: FuncId,
    list_filter_id: FuncId,
    list_map_id: FuncId,
    list_to_json_id: FuncId,

    // Variants (Option / Result)
    variant_tag_id: FuncId,
    variant_val_id: FuncId,
    option_some_id: FuncId,
    option_none_id: FuncId,
    result_ok_id: FuncId,
    result_err_id: FuncId,

    // HTTP Server
    http_new_serve_mux_id: FuncId,
    http_mux_use_id: FuncId,
    http_mux_handle_id: FuncId,
    http_listen_and_serve_id: FuncId,
    http_path_value_id: FuncId,
    http_query_id: FuncId,
    http_req_method_id: FuncId,
    http_req_url_id: FuncId,
    http_req_body_id: FuncId,
    http_set_header_id: FuncId,
    http_json_id: FuncId,
    http_error_id: FuncId,
    http_parse_json_id: FuncId,

    // Value Arithmetic & Comparison Helpers
    val_eq_id: FuncId,
    val_add_id: FuncId,
    val_sub_id: FuncId,
    val_mul_id: FuncId,
    val_div_id: FuncId,
    val_to_str_id: FuncId,

    lambda_counter: usize,
    fiber_counter: usize,
}

fn declare_import_fn(
    module: &mut ObjectModule,
    name: &str,
    params: usize,
    has_ret: bool,
) -> FuncId {
    let call_conv = module.target_config().default_call_conv;
    let mut sig = Signature::new(call_conv);
    for _ in 0..params {
        sig.params.push(AbiParam::new(types::I64));
    }
    if has_ret {
        sig.returns.push(AbiParam::new(types::I64));
    }
    module
        .declare_function(name, Linkage::Import, &sig)
        .unwrap()
}

fn collect_pattern_names(pattern: &Pattern, names: &mut Vec<String>) {
    match pattern {
        Pattern::Variable(name) => names.push(name.clone()),
        Pattern::Constructor { patterns, .. } => {
            for p in patterns {
                collect_pattern_names(p, names);
            }
        }
        Pattern::Record { fields, .. } => {
            for (_, p) in fields {
                collect_pattern_names(p, names);
            }
        }
        Pattern::Tuple(patterns) => {
            for p in patterns {
                collect_pattern_names(p, names);
            }
        }
        Pattern::List { items, rest } => {
            for p in items {
                collect_pattern_names(p, names);
            }
            if let Some(r) = rest {
                collect_pattern_names(r, names);
            }
        }
        _ => {}
    }
}

fn collect_let_names(expr: &Expr, names: &mut Vec<String>) {
    match expr {
        Expr::Block(stmts) => {
            for stmt in stmts {
                match stmt {
                    Statement::Let { name, value, .. } => {
                        names.push(name.clone());
                        collect_let_names(value, names);
                    }
                    Statement::LetPattern { pattern, value, .. } => {
                        collect_pattern_names(pattern, names);
                        collect_let_names(value, names);
                    }
                    Statement::Assign { target, value } => {
                        collect_let_names(target, names);
                        collect_let_names(value, names);
                    }
                    Statement::Defer(e) | Statement::ErrDefer(e) => collect_let_names(e, names),
                    Statement::Expr(e) => collect_let_names(e, names),
                    Statement::Return(Some(e)) => collect_let_names(e, names),
                    _ => {}
                }
            }
        }
        Expr::Binary { left, right, .. } | Expr::Pipeline { left, right } => {
            collect_let_names(left, names);
            collect_let_names(right, names);
        }
        Expr::Unary { expr, .. }
        | Expr::Async(expr)
        | Expr::Await(expr)
        | Expr::Spawn(expr)
        | Expr::ChanRecv(expr)
        | Expr::Try(expr) => {
            collect_let_names(expr, names);
        }
        Expr::FunctionCall { callee, args } => {
            collect_let_names(callee, names);
            for a in args {
                collect_let_names(a, names);
            }
        }
        Expr::MemberAccess { object, .. } => {
            collect_let_names(object, names);
        }
        Expr::IndexAccess { object, index } => {
            collect_let_names(object, names);
            collect_let_names(index, names);
        }
        Expr::SliceAccess {
            object,
            low,
            high,
            max,
        } => {
            collect_let_names(object, names);
            if let Some(l) = low {
                collect_let_names(l, names);
            }
            if let Some(h) = high {
                collect_let_names(h, names);
            }
            if let Some(m) = max {
                collect_let_names(m, names);
            }
        }
        Expr::RecordLiteral { fields, spread } => {
            for (_, f_expr) in fields {
                collect_let_names(f_expr, names);
            }
            if let Some(s) = spread {
                collect_let_names(s, names);
            }
        }
        Expr::TupleLiteral(elems) | Expr::ListLiteral(elems) => {
            for el in elems {
                collect_let_names(el, names);
            }
        }
        Expr::Lambda { body, .. } => {
            collect_let_names(body, names);
        }
        Expr::Match { subject, arms } => {
            collect_let_names(subject, names);
            for arm in arms {
                collect_pattern_names(&arm.pattern, names);
                collect_let_names(&arm.body, names);
            }
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            collect_let_names(condition, names);
            collect_let_names(then_branch, names);
            if let Some(eb) = else_branch {
                collect_let_names(eb, names);
            }
        }
        Expr::ChanSend { channel, value } => {
            collect_let_names(channel, names);
            collect_let_names(value, names);
        }
        Expr::ConstructorCall { args, .. } => {
            for a in args {
                collect_let_names(a, names);
            }
        }
        Expr::While {
            condition, body, ..
        } => {
            collect_let_names(condition, names);
            collect_let_names(body, names);
        }
        Expr::ForIn {
            var_name,
            iterable,
            body,
            ..
        } => {
            names.push(var_name.clone());
            collect_let_names(iterable, names);
            collect_let_names(body, names);
        }
        Expr::Select { arms, default } => {
            for arm in arms {
                match &arm.kind {
                    SelectArmKind::Recv { binding, channel } => {
                        if let Some(b) = binding {
                            names.push(b.clone());
                        }
                        collect_let_names(channel, names);
                    }
                    SelectArmKind::Send { channel, value } => {
                        collect_let_names(channel, names);
                        collect_let_names(value, names);
                    }
                    SelectArmKind::Timeout(t) => {
                        collect_let_names(t, names);
                    }
                }
                collect_let_names(&arm.body, names);
            }
            if let Some(d) = default {
                collect_let_names(d, names);
            }
        }
        _ => {}
    }
}

fn is_block_terminated(builder: &FunctionBuilder, block: Block) -> bool {
    builder.func.layout.last_inst(block).map_or(false, |inst| {
        let op = builder.func.dfg.insts[inst].opcode();
        op.is_branch() || op.is_return()
    })
}

impl CraneliftBackend {
    pub fn new() -> Result<Self, String> {
        Self::with_target(None)
    }

    pub fn with_target(target: Option<&str>) -> Result<Self, String> {
        Self::with_options(target, false)
    }

    pub fn with_options(target: Option<&str>, release: bool) -> Result<Self, String> {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "true").unwrap();
        if release {
            flag_builder.set("opt_level", "speed").unwrap();
            flag_builder.set("enable_simd", "true").unwrap();
        }

        let triple = if let Some(t) = target {
            match t {
                "x86_64-linux" | "linux/amd64" | "linux-x64" => "x86_64-unknown-linux-musl"
                    .parse()
                    .map_err(|e| format!("Invalid target: {}", e))?,
                "aarch64-linux" | "linux/arm64" | "linux-arm64" => "aarch64-unknown-linux-musl"
                    .parse()
                    .map_err(|e| format!("Invalid target: {}", e))?,
                "x86_64-macos" | "darwin/amd64" | "darwin-x64" => "x86_64-apple-darwin"
                    .parse()
                    .map_err(|e| format!("Invalid target: {}", e))?,
                "aarch64-macos" | "darwin/arm64" | "darwin-arm64" => "aarch64-apple-darwin"
                    .parse()
                    .map_err(|e| format!("Invalid target: {}", e))?,
                "x86_64-windows" | "windows/amd64" | "windows-x64" => "x86_64-pc-windows-gnu"
                    .parse()
                    .map_err(|e| format!("Invalid target: {}", e))?,
                other => other
                    .parse()
                    .map_err(|e| format!("Invalid target triple '{}': {}", other, e))?,
            }
        } else {
            target_lexicon::Triple::host()
        };
        let isa_builder = isa::lookup(triple).map_err(|e| format!("ISA lookup failed: {}", e))?;
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .map_err(|e| format!("ISA finish failed: {}", e))?;

        let obj_builder = ObjectBuilder::new(isa, "aura_module", default_libcall_names())
            .map_err(|e| format!("ObjectBuilder error: {}", e))?;
        let mut module = ObjectModule::new(obj_builder);

        // Core runtime
        let rt_start_id = declare_import_fn(&mut module, "aura_rt_start", 1, false);
        let fiber_spawn_id = declare_import_fn(&mut module, "aura_fiber_spawn", 2, true);
        let fiber_yield_id = declare_import_fn(&mut module, "aura_fiber_yield", 0, false);
        let chan_new_id = declare_import_fn(&mut module, "aura_chan_new", 1, true);
        let chan_send_id = declare_import_fn(&mut module, "aura_chan_send", 2, false);
        let chan_recv_id = declare_import_fn(&mut module, "aura_chan_recv", 1, true);
        let gc_alloc_id = declare_import_fn(&mut module, "aura_gc_alloc", 2, true);
        let gc_safepoint_id = declare_import_fn(&mut module, "aura_gc_safepoint", 0, false);
        let gc_collect_id = declare_import_fn(&mut module, "aura_gc_collect", 0, false);

        // Printing
        let println_any_id = declare_import_fn(&mut module, "aura_println_any", 1, false);
        let println_str_id = declare_import_fn(&mut module, "aura_println_str", 2, false);

        // Synchronization
        let mutex_new_id = declare_import_fn(&mut module, "aura_mutex_new", 0, true);
        let mutex_lock_id = declare_import_fn(&mut module, "aura_mutex_lock", 1, false);
        let mutex_unlock_id = declare_import_fn(&mut module, "aura_mutex_unlock", 1, false);

        // OS & Utilities
        let os_env_id = declare_import_fn(&mut module, "aura_os_env", 2, true);
        let os_hostname_id = declare_import_fn(&mut module, "aura_os_hostname", 0, true);
        let process_uptime_id = declare_import_fn(&mut module, "aura_process_uptime", 0, true);
        let time_now_id = declare_import_fn(&mut module, "aura_time_now", 0, true);
        let sleep_ms_id = declare_import_fn(&mut module, "aura_sleep_ms", 1, false);
        let parse_float_id = declare_import_fn(&mut module, "aura_parse_float", 1, true);

        // Strings
        let string_new_id = declare_import_fn(&mut module, "aura_string_new", 2, true);
        let string_concat_id = declare_import_fn(&mut module, "aura_string_concat", 2, true);
        let string_eq_id = declare_import_fn(&mut module, "aura_string_eq", 2, true);
        let string_to_lower_id = declare_import_fn(&mut module, "aura_string_to_lower", 1, true);
        let string_contains_id = declare_import_fn(&mut module, "aura_string_contains", 2, true);
        let string_len_id = declare_import_fn(&mut module, "aura_string_len", 1, true);
        let string_cstr_id = declare_import_fn(&mut module, "aura_string_cstr", 1, true);

        // Records / Structs
        let record_new_id = declare_import_fn(&mut module, "aura_record_new", 0, true);
        let record_set_id = declare_import_fn(&mut module, "aura_record_set", 4, false);
        let record_get_id = declare_import_fn(&mut module, "aura_record_get", 3, true);
        let record_to_json_id = declare_import_fn(&mut module, "aura_record_to_json", 1, true);

        // Lists
        let list_new_id = declare_import_fn(&mut module, "aura_list_new", 1, true);
        let list_push_id = declare_import_fn(&mut module, "aura_list_push", 2, false);
        let list_get_id = declare_import_fn(&mut module, "aura_list_get", 2, true);
        let list_set_id = declare_import_fn(&mut module, "aura_list_set", 3, false);
        let list_len_id = declare_import_fn(&mut module, "aura_list_len", 1, true);
        let list_cap_id = declare_import_fn(&mut module, "aura_list_cap", 1, true);
        let list_slice_id = declare_import_fn(&mut module, "aura_list_slice", 3, true);
        let list_filter_id = declare_import_fn(&mut module, "aura_list_filter", 2, true);
        let list_map_id = declare_import_fn(&mut module, "aura_list_map", 2, true);
        let list_to_json_id = declare_import_fn(&mut module, "aura_list_to_json", 1, true);

        // Variants (Option / Result)
        let variant_tag_id = declare_import_fn(&mut module, "aura_variant_tag", 1, true);
        let variant_val_id = declare_import_fn(&mut module, "aura_variant_val", 1, true);
        let option_some_id = declare_import_fn(&mut module, "aura_option_some", 1, true);
        let option_none_id = declare_import_fn(&mut module, "aura_option_none", 0, true);
        let result_ok_id = declare_import_fn(&mut module, "aura_result_ok", 1, true);
        let result_err_id = declare_import_fn(&mut module, "aura_result_err", 1, true);

        // HTTP Server
        let http_new_serve_mux_id =
            declare_import_fn(&mut module, "aura_http_new_serve_mux", 0, true);
        let http_mux_use_id = declare_import_fn(&mut module, "aura_http_mux_use", 2, false);
        let http_mux_handle_id = declare_import_fn(&mut module, "aura_http_mux_handle", 6, false);
        let http_listen_and_serve_id =
            declare_import_fn(&mut module, "aura_http_listen_and_serve", 3, false);
        let http_path_value_id = declare_import_fn(&mut module, "aura_http_path_value", 3, true);
        let http_query_id = declare_import_fn(&mut module, "aura_http_query", 3, true);
        let http_req_method_id = declare_import_fn(&mut module, "aura_http_req_method", 1, true);
        let http_req_url_id = declare_import_fn(&mut module, "aura_http_req_url", 1, true);
        let http_req_body_id = declare_import_fn(&mut module, "aura_http_req_body", 1, true);
        let http_set_header_id = declare_import_fn(&mut module, "aura_http_set_header", 5, false);
        let http_json_id = declare_import_fn(&mut module, "aura_http_json", 3, false);
        let http_error_id = declare_import_fn(&mut module, "aura_http_error", 4, false);
        let http_parse_json_id = declare_import_fn(&mut module, "aura_http_parse_json", 1, true);

        // Value Arithmetic & Comparison Helpers
        let val_eq_id = declare_import_fn(&mut module, "aura_val_eq", 2, true);
        let val_add_id = declare_import_fn(&mut module, "aura_val_add", 2, true);
        let val_sub_id = declare_import_fn(&mut module, "aura_val_sub", 2, true);
        let val_mul_id = declare_import_fn(&mut module, "aura_val_mul", 2, true);
        let val_div_id = declare_import_fn(&mut module, "aura_val_div", 2, true);
        let val_to_str_id = declare_import_fn(&mut module, "aura_val_to_str", 1, true);

        Ok(CraneliftBackend {
            module,
            func_ids: HashMap::new(),
            globals: HashMap::new(),
            rt_start_id,
            fiber_spawn_id,
            fiber_yield_id,
            chan_new_id,
            chan_send_id,
            chan_recv_id,
            gc_alloc_id,
            gc_safepoint_id,
            gc_collect_id,
            println_any_id,
            println_str_id,
            mutex_new_id,
            mutex_lock_id,
            mutex_unlock_id,
            os_env_id,
            os_hostname_id,
            process_uptime_id,
            time_now_id,
            sleep_ms_id,
            parse_float_id,
            string_new_id,
            string_concat_id,
            string_eq_id,
            string_to_lower_id,
            string_contains_id,
            string_len_id,
            string_cstr_id,
            record_new_id,
            record_set_id,
            record_get_id,
            record_to_json_id,
            list_new_id,
            list_push_id,
            list_get_id,
            list_set_id,
            list_len_id,
            list_cap_id,
            list_slice_id,
            list_filter_id,
            list_map_id,
            list_to_json_id,
            variant_tag_id,
            variant_val_id,
            option_some_id,
            option_none_id,
            result_ok_id,
            result_err_id,
            http_new_serve_mux_id,
            http_mux_use_id,
            http_mux_handle_id,
            http_listen_and_serve_id,
            http_path_value_id,
            http_query_id,
            http_req_method_id,
            http_req_url_id,
            http_req_body_id,
            http_set_header_id,
            http_json_id,
            http_error_id,
            http_parse_json_id,
            val_eq_id,
            val_add_id,
            val_sub_id,
            val_mul_id,
            val_div_id,
            val_to_str_id,
            lambda_counter: 0,
            fiber_counter: 0,
        })
    }

    pub fn compile_module(mut self, aura_module: &Module) -> Result<Vec<u8>, String> {
        let call_conv = self.module.target_config().default_call_conv;

        let mut top_level_stmts = Vec::new();
        let mut has_main = false;
        let mut all_let_names = Vec::new();

        for item in &aura_module.items {
            match item {
                Item::Function(f) => {
                    if f.is_async {
                        return Err(
                            "Async functions are compiled via the Go native backend".to_string()
                        );
                    }
                    if f.name == "main" {
                        has_main = true;
                    }
                    collect_let_names(&f.body, &mut all_let_names);
                }
                Item::Extern(ext) => {
                    for f in &ext.functions {
                        let has_ret = !matches!(f.return_type, Type::Unit);
                        let fid =
                            declare_import_fn(&mut self.module, &f.name, f.params.len(), has_ret);
                        self.func_ids.insert(f.name.clone(), fid);
                    }
                }
                Item::Statement(stmt) => {
                    match stmt {
                        Statement::Let { name, value, .. } => {
                            all_let_names.push(name.clone());
                            collect_let_names(value, &mut all_let_names);
                        }
                        Statement::LetPattern { pattern, value, .. } => {
                            collect_pattern_names(pattern, &mut all_let_names);
                            collect_let_names(value, &mut all_let_names);
                        }
                        Statement::Assign { target, value } => {
                            collect_let_names(target, &mut all_let_names);
                            collect_let_names(value, &mut all_let_names);
                        }
                        Statement::Defer(e) | Statement::ErrDefer(e) => {
                            collect_let_names(e, &mut all_let_names);
                        }
                        Statement::Expr(e) => {
                            collect_let_names(e, &mut all_let_names);
                        }
                        Statement::Return(Some(e)) => {
                            collect_let_names(e, &mut all_let_names);
                        }
                        _ => {}
                    }
                    top_level_stmts.push(stmt.clone());
                }
                _ => {}
            }
        }

        for name in all_let_names {
            if !name.is_empty() && name != "null" && !self.globals.contains_key(&name) {
                let data_id = self
                    .module
                    .declare_data(&name, Linkage::Local, true, false)
                    .map_err(|e| format!("Error declaring global '{}': {}", name, e))?;
                let mut desc = cranelift_module::DataDescription::new();
                desc.define_zeroinit(8); // 8 bytes
                self.module
                    .define_data(data_id, &desc)
                    .map_err(|e| format!("Error defining global '{}': {}", name, e))?;
                self.globals.insert(name, data_id);
            }
        }

        let synthetic_main = if !has_main && !top_level_stmts.is_empty() {
            Some(FunctionDecl {
                name: "main".to_string(),
                receiver: None,
                is_async: false,
                is_exported: true,
                type_params: vec![],
                params: vec![],
                return_type: Some(Type::Unit),
                body: Expr::Block(top_level_stmts),
            })
        } else {
            None
        };

        // 1. Declare all functions
        for item in &aura_module.items {
            if let Item::Function(f) = item {
                let mut sig = Signature::new(call_conv);
                for _ in &f.params {
                    sig.params.push(AbiParam::new(types::I64));
                }
                sig.returns.push(AbiParam::new(types::I64));

                let symbol_name = if f.name == "main" {
                    "aura_user_main".to_string()
                } else {
                    f.name.clone()
                };

                let func_id = self
                    .module
                    .declare_function(&symbol_name, Linkage::Export, &sig)
                    .map_err(|e| format!("Error declaring function '{}': {}", symbol_name, e))?;

                self.func_ids.insert(f.name.clone(), func_id);
            }
        }

        if let Some(ref f) = synthetic_main {
            let mut sig = Signature::new(call_conv);
            sig.returns.push(AbiParam::new(types::I64));
            let func_id = self
                .module
                .declare_function("aura_user_main", Linkage::Export, &sig)
                .map_err(|e| format!("Error declaring synthetic main: {}", e))?;
            self.func_ids.insert(f.name.clone(), func_id);
        }

        // 2. Compile function bodies
        for item in &aura_module.items {
            if let Item::Function(f) = item {
                self.compile_function(f)?;
            }
        }

        if let Some(ref f) = synthetic_main {
            self.compile_function(f)?;
        }

        // 3. Emit standard C entrypoint main() that boots Aura runtime
        if self.func_ids.contains_key("main") {
            self.emit_c_main_entrypoint()?;
        } else {
            return Err(
                "No 'main' function or top-level statements found to generate executable."
                    .to_string(),
            );
        }

        let product = self.module.finish();
        let bytes = product
            .emit()
            .map_err(|e| format!("Object emission error: {}", e))?;
        Ok(bytes)
    }

    fn compile_function(&mut self, f: &FunctionDecl) -> Result<(), String> {
        let func_id = self.func_ids[&f.name];
        let target_config = self.module.target_config();
        let mut ctx = self.module.make_context();
        ctx.func.signature = Signature::new(target_config.default_call_conv);

        for _ in &f.params {
            ctx.func.signature.params.push(AbiParam::new(types::I64));
        }
        ctx.func.signature.returns.push(AbiParam::new(types::I64));

        let mut fn_builder_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut fn_builder_ctx);
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        let mut vars: HashMap<String, (Variable, types::Type)> = HashMap::new();
        let mut defers: Vec<(Expr, bool)> = Vec::new();

        for (i, p) in f.params.iter().enumerate() {
            let var = builder.declare_var(types::I64);
            let val = builder.block_params(entry_block)[i];
            builder.def_var(var, val);
            vars.insert(p.name.clone(), (var, types::I64));
            if let Some(&data_id) = self.globals.get(&p.name) {
                let local_data = self.module.declare_data_in_func(data_id, builder.func);
                let addr = builder.ins().symbol_value(types::I64, local_data);
                builder.ins().store(MemFlagsData::trusted(), val, addr, 0);
            }
        }

        // Compile body
        let ret_val = self.compile_expr(&mut builder, &f.body, &mut vars, &mut defers, None)?;

        // Run any remaining deferred statements before function exit
        self.compile_defers(&mut builder, &defers, &mut vars, None, ret_val)?;

        let should_return = if let Some(cur) = builder.current_block() {
            !is_block_terminated(&builder, cur)
        } else {
            false
        };
        if should_return && !builder.is_unreachable() {
            builder.ins().return_(&[ret_val]);
        }
        builder.finalize(target_config);

        self.module
            .define_function(func_id, &mut ctx)
            .map_err(|e| format!("Error defining function '{}': {}", f.name, e))?;

        Ok(())
    }

    fn get_string_ptr_and_len(
        &mut self,
        builder: &mut FunctionBuilder,
        str_val: Value,
    ) -> (Value, Value) {
        let local_cstr = self
            .module
            .declare_func_in_func(self.string_cstr_id, builder.func);
        let call_cstr = builder.ins().call(local_cstr, &[str_val]);
        let ptr_val = builder.inst_results(call_cstr)[0];

        let local_len = self
            .module
            .declare_func_in_func(self.string_len_id, builder.func);
        let call_len = builder.ins().call(local_len, &[str_val]);
        let len_val = builder.inst_results(call_len)[0];

        (ptr_val, len_val)
    }

    fn emit_string_literal(
        &mut self,
        builder: &mut FunctionBuilder,
        s: &str,
    ) -> Result<Value, String> {
        let data_id = self
            .module
            .declare_anonymous_data(true, false)
            .map_err(|e| format!("Data alloc error: {}", e))?;
        let mut data_ctx = cranelift_module::DataDescription::new();
        data_ctx.define(s.as_bytes().to_vec().into_boxed_slice());
        self.module
            .define_data(data_id, &data_ctx)
            .map_err(|e| format!("Data define error: {}", e))?;

        let global_val = self.module.declare_data_in_func(data_id, builder.func);
        let ptr = builder.ins().symbol_value(types::I64, global_val);
        let len = builder.ins().iconst(types::I64, s.len() as i64);

        let local_func = self
            .module
            .declare_func_in_func(self.string_new_id, builder.func);
        let call = builder.ins().call(local_func, &[ptr, len]);
        Ok(builder.inst_results(call)[0])
    }

    fn compile_defers(
        &mut self,
        builder: &mut FunctionBuilder,
        defers: &[(Expr, bool)],
        vars: &mut HashMap<String, (Variable, types::Type)>,
        loop_ctx: Option<(cranelift_codegen::ir::Block, cranelift_codegen::ir::Block)>,
        ret_val: Value,
    ) -> Result<(), String> {
        let has_errdefer = defers.iter().any(|(_, is_err)| *is_err);
        if !has_errdefer {
            for (d, _) in defers.iter().rev() {
                let mut sub_defers = Vec::new();
                self.compile_expr(builder, d, vars, &mut sub_defers, loop_ctx)?;
            }
            return Ok(());
        }

        let local_tag = self
            .module
            .declare_func_in_func(self.variant_tag_id, builder.func);
        let call_tag = builder.ins().call(local_tag, &[ret_val]);
        let tag_val = builder.inst_results(call_tag)[0];
        let tag_err = builder.ins().iconst(types::I64, 2);
        let is_err = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            tag_val,
            tag_err,
        );

        for (d, is_errdefer) in defers.iter().rev() {
            if !*is_errdefer {
                let mut sub_defers = Vec::new();
                self.compile_expr(builder, d, vars, &mut sub_defers, loop_ctx)?;
            } else {
                let err_exec = builder.create_block();
                let cont = builder.create_block();
                builder.ins().brif(is_err, err_exec, &[], cont, &[]);

                builder.switch_to_block(err_exec);
                builder.seal_block(err_exec);
                let mut sub_defers = Vec::new();
                self.compile_expr(builder, d, vars, &mut sub_defers, loop_ctx)?;
                if !builder.is_unreachable() {
                    builder.ins().jump(cont, &[]);
                }

                builder.switch_to_block(cont);
                builder.seal_block(cont);
            }
        }
        Ok(())
    }

    fn compile_expr(
        &mut self,
        builder: &mut FunctionBuilder,
        expr: &Expr,
        vars: &mut HashMap<String, (Variable, types::Type)>,
        defers: &mut Vec<(Expr, bool)>,
        loop_ctx: Option<(cranelift_codegen::ir::Block, cranelift_codegen::ir::Block)>,
    ) -> Result<Value, String> {
        match expr {
            Expr::Literal(Literal::Int(n)) => Ok(builder.ins().iconst(types::I64, *n)),
            Expr::Literal(Literal::Float(f)) => {
                Ok(builder.ins().iconst(types::I64, f.to_bits() as i64))
            }
            Expr::Literal(Literal::Bool(b)) => {
                let v = if *b { 1 } else { 0 };
                Ok(builder.ins().iconst(types::I64, v))
            }
            Expr::Literal(Literal::Unit) => Ok(builder.ins().iconst(types::I64, 0)),
            Expr::Literal(Literal::String(s)) => self.emit_string_literal(builder, s),
            Expr::Literal(Literal::TemplateString(segments)) => {
                let mut current = self.emit_string_literal(builder, "")?;
                for seg in segments {
                    let seg_val = match seg {
                        TemplateSegment::Text(t) => self.emit_string_literal(builder, t)?,
                        TemplateSegment::Expr(e) => {
                            let ev = self.compile_expr(builder, e, vars, defers, loop_ctx)?;
                            let to_str = self
                                .module
                                .declare_func_in_func(self.val_to_str_id, builder.func);
                            let call = builder.ins().call(to_str, &[ev]);
                            builder.inst_results(call)[0]
                        }
                    };
                    let local_concat = self
                        .module
                        .declare_func_in_func(self.string_concat_id, builder.func);
                    let call = builder.ins().call(local_concat, &[current, seg_val]);
                    current = builder.inst_results(call)[0];
                }
                Ok(current)
            }
            Expr::Identifier(name) => {
                if name == "null" {
                    return Ok(builder.ins().iconst(types::I64, 0));
                }
                if let Some((var, _)) = vars.get(name) {
                    Ok(builder.use_var(*var))
                } else if let Some(&data_id) = self.globals.get(name) {
                    let local_data = self.module.declare_data_in_func(data_id, builder.func);
                    let addr = builder.ins().symbol_value(types::I64, local_data);
                    Ok(builder
                        .ins()
                        .load(types::I64, MemFlagsData::trusted(), addr, 0))
                } else {
                    Err(format!("Undefined variable '{}'", name))
                }
            }
            Expr::ListLiteral(elements) => {
                let cap = builder.ins().iconst(types::I64, elements.len() as i64);
                let local_new = self
                    .module
                    .declare_func_in_func(self.list_new_id, builder.func);
                let call = builder.ins().call(local_new, &[cap]);
                let list_ptr = builder.inst_results(call)[0];

                let local_push = self
                    .module
                    .declare_func_in_func(self.list_push_id, builder.func);
                for el in elements {
                    let el_val = self.compile_expr(builder, el, vars, defers, loop_ctx)?;
                    builder.ins().call(local_push, &[list_ptr, el_val]);
                }
                Ok(list_ptr)
            }
            Expr::RecordLiteral { fields, .. } => {
                let local_new = self
                    .module
                    .declare_func_in_func(self.record_new_id, builder.func);
                let call = builder.ins().call(local_new, &[]);
                let rec_ptr = builder.inst_results(call)[0];

                let local_set = self
                    .module
                    .declare_func_in_func(self.record_set_id, builder.func);
                for (k, v) in fields {
                    let k_val = self.emit_string_literal(builder, k)?;
                    let (ptr_val, len_val) = self.get_string_ptr_and_len(builder, k_val);

                    let v_val = self.compile_expr(builder, v, vars, defers, loop_ctx)?;
                    builder
                        .ins()
                        .call(local_set, &[rec_ptr, ptr_val, len_val, v_val]);
                }
                Ok(rec_ptr)
            }
            Expr::IndexAccess { object, index } => {
                let obj_val = self.compile_expr(builder, object, vars, defers, loop_ctx)?;
                let idx_val = self.compile_expr(builder, index, vars, defers, loop_ctx)?;

                let res_var = builder.declare_var(types::I64);
                let zero = builder.ins().iconst(types::I64, 0);
                let not_null = builder.ins().icmp(
                    cranelift_codegen::ir::condcodes::IntCC::NotEqual,
                    obj_val,
                    zero,
                );
                let check_block = builder.create_block();
                let slow_block = builder.create_block();
                let fast_block = builder.create_block();
                let merge_block = builder.create_block();

                builder
                    .ins()
                    .brif(not_null, check_block, &[], slow_block, &[]);

                builder.switch_to_block(check_block);
                builder.seal_block(check_block);
                let len_val = builder
                    .ins()
                    .load(types::I64, MemFlagsData::trusted(), obj_val, 16);
                let in_bounds = builder.ins().icmp(
                    cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThan,
                    idx_val,
                    len_val,
                );
                builder
                    .ins()
                    .brif(in_bounds, fast_block, &[], slow_block, &[]);

                builder.switch_to_block(fast_block);
                builder.seal_block(fast_block);
                let items_ptr = builder
                    .ins()
                    .load(types::I64, MemFlagsData::trusted(), obj_val, 8);
                let offset = builder.ins().ishl_imm_u(idx_val, 3);
                let addr = builder.ins().iadd(items_ptr, offset);
                let elem = builder
                    .ins()
                    .load(types::I64, MemFlagsData::trusted(), addr, 0);
                builder.def_var(res_var, elem);
                builder.ins().jump(merge_block, &[]);

                builder.switch_to_block(slow_block);
                builder.seal_block(slow_block);
                let local_get = self
                    .module
                    .declare_func_in_func(self.list_get_id, builder.func);
                let call = builder.ins().call(local_get, &[obj_val, idx_val]);
                let fallback_res = builder.inst_results(call)[0];
                builder.def_var(res_var, fallback_res);
                builder.ins().jump(merge_block, &[]);

                builder.switch_to_block(merge_block);
                builder.seal_block(merge_block);
                Ok(builder.use_var(res_var))
            }
            Expr::SliceAccess {
                object, low, high, ..
            } => {
                let obj_val = self.compile_expr(builder, object, vars, defers, loop_ctx)?;
                let low_val = match low {
                    Some(l) => self.compile_expr(builder, l, vars, defers, loop_ctx)?,
                    None => builder.ins().iconst(types::I64, 0),
                };
                let high_val = match high {
                    Some(h) => self.compile_expr(builder, h, vars, defers, loop_ctx)?,
                    None => builder.ins().iconst(types::I64, -1),
                };
                let local_fn = self
                    .module
                    .declare_func_in_func(self.list_slice_id, builder.func);
                let call = builder.ins().call(local_fn, &[obj_val, low_val, high_val]);
                Ok(builder.inst_results(call)[0])
            }
            Expr::MemberAccess { object, member } => {
                if let Expr::Identifier(ref name) = **object {
                    if name == "http" {
                        match member.as_str() {
                            "StatusOK" => return Ok(builder.ins().iconst(types::I64, 200)),
                            "StatusCreated" => return Ok(builder.ins().iconst(types::I64, 201)),
                            "StatusNoContent" => return Ok(builder.ins().iconst(types::I64, 204)),
                            "StatusBadRequest" => return Ok(builder.ins().iconst(types::I64, 400)),
                            "StatusNotFound" => return Ok(builder.ins().iconst(types::I64, 404)),
                            "StatusInternalServerError" => {
                                return Ok(builder.ins().iconst(types::I64, 500));
                            }
                            _ => {}
                        }
                    }
                }

                let obj_val = self.compile_expr(builder, object, vars, defers, loop_ctx)?;

                if member == "length" || member == "len" {
                    let local_len = self
                        .module
                        .declare_func_in_func(self.list_len_id, builder.func);
                    let call = builder.ins().call(local_len, &[obj_val]);
                    return Ok(builder.inst_results(call)[0]);
                }

                if member == "capacity" || member == "cap" {
                    let local_cap = self
                        .module
                        .declare_func_in_func(self.list_cap_id, builder.func);
                    let call = builder.ins().call(local_cap, &[obj_val]);
                    return Ok(builder.inst_results(call)[0]);
                }

                if member == "method" {
                    let local_m = self
                        .module
                        .declare_func_in_func(self.http_req_method_id, builder.func);
                    let call = builder.ins().call(local_m, &[obj_val]);
                    return Ok(builder.inst_results(call)[0]);
                }

                if member == "url" {
                    let local_u = self
                        .module
                        .declare_func_in_func(self.http_req_url_id, builder.func);
                    let call = builder.ins().call(local_u, &[obj_val]);
                    return Ok(builder.inst_results(call)[0]);
                }

                // Default record property access
                let k_val = self.emit_string_literal(builder, member)?;
                let (ptr_val, len_val) = self.get_string_ptr_and_len(builder, k_val);

                let local_get = self
                    .module
                    .declare_func_in_func(self.record_get_id, builder.func);
                let call = builder.ins().call(local_get, &[obj_val, ptr_val, len_val]);
                Ok(builder.inst_results(call)[0])
            }
            Expr::Binary { op, left, right } => {
                let l = self.compile_expr(builder, left, vars, defers, loop_ctx)?;
                let r = self.compile_expr(builder, right, vars, defers, loop_ctx)?;
                match op {
                    BinOp::Add => {
                        let is_str = matches!(
                            **left,
                            Expr::Literal(Literal::String(_))
                                | Expr::Literal(Literal::TemplateString(_))
                        ) || matches!(
                            **right,
                            Expr::Literal(Literal::String(_))
                                | Expr::Literal(Literal::TemplateString(_))
                        );
                        if is_str {
                            let local_add = self
                                .module
                                .declare_func_in_func(self.val_add_id, builder.func);
                            let call = builder.ins().call(local_add, &[l, r]);
                            Ok(builder.inst_results(call)[0])
                        } else {
                            Ok(builder.ins().iadd(l, r))
                        }
                    }
                    BinOp::Sub => Ok(builder.ins().isub(l, r)),
                    BinOp::Mul => Ok(builder.ins().imul(l, r)),
                    BinOp::Div => {
                        let res_var = builder.declare_var(types::I64);
                        let zero = builder.ins().iconst(types::I64, 0);
                        let is_zero = builder.ins().icmp(
                            cranelift_codegen::ir::condcodes::IntCC::Equal,
                            r,
                            zero,
                        );
                        let safe_div_block = builder.create_block();
                        let zero_div_block = builder.create_block();
                        let merge_block = builder.create_block();

                        builder
                            .ins()
                            .brif(is_zero, zero_div_block, &[], safe_div_block, &[]);

                        builder.switch_to_block(safe_div_block);
                        builder.seal_block(safe_div_block);
                        let div_res = builder.ins().sdiv(l, r);
                        builder.def_var(res_var, div_res);
                        builder.ins().jump(merge_block, &[]);

                        builder.switch_to_block(zero_div_block);
                        builder.seal_block(zero_div_block);
                        builder.def_var(res_var, zero);
                        builder.ins().jump(merge_block, &[]);

                        builder.switch_to_block(merge_block);
                        builder.seal_block(merge_block);
                        Ok(builder.use_var(res_var))
                    }
                    BinOp::Mod => Ok(builder.ins().srem(l, r)),
                    BinOp::Equal => {
                        let local_eq = self
                            .module
                            .declare_func_in_func(self.val_eq_id, builder.func);
                        let call = builder.ins().call(local_eq, &[l, r]);
                        Ok(builder.inst_results(call)[0])
                    }
                    BinOp::NotEqual => {
                        let local_eq = self
                            .module
                            .declare_func_in_func(self.val_eq_id, builder.func);
                        let call = builder.ins().call(local_eq, &[l, r]);
                        let res = builder.inst_results(call)[0];
                        let one = builder.ins().iconst(types::I64, 1);
                        Ok(builder.ins().bxor(res, one))
                    }
                    BinOp::LessThan => {
                        let cmp = builder.ins().icmp(
                            cranelift_codegen::ir::condcodes::IntCC::SignedLessThan,
                            l,
                            r,
                        );
                        Ok(builder.ins().uextend(types::I64, cmp))
                    }
                    BinOp::LessEqual => {
                        let cmp = builder.ins().icmp(
                            cranelift_codegen::ir::condcodes::IntCC::SignedLessThanOrEqual,
                            l,
                            r,
                        );
                        Ok(builder.ins().uextend(types::I64, cmp))
                    }
                    BinOp::GreaterThan => {
                        let cmp = builder.ins().icmp(
                            cranelift_codegen::ir::condcodes::IntCC::SignedGreaterThan,
                            l,
                            r,
                        );
                        Ok(builder.ins().uextend(types::I64, cmp))
                    }
                    BinOp::GreaterEqual => {
                        let cmp = builder.ins().icmp(
                            cranelift_codegen::ir::condcodes::IntCC::SignedGreaterThanOrEqual,
                            l,
                            r,
                        );
                        Ok(builder.ins().uextend(types::I64, cmp))
                    }
                    BinOp::And => Ok(builder.ins().band(l, r)),
                    BinOp::Or => Ok(builder.ins().bor(l, r)),
                }
            }
            Expr::Unary { op, expr } => {
                let val = self.compile_expr(builder, expr, vars, defers, loop_ctx)?;
                match op {
                    UnOp::Negate => Ok(builder.ins().ineg(val)),
                    UnOp::Not => {
                        let one = builder.ins().iconst(types::I64, 1);
                        Ok(builder.ins().bxor(val, one))
                    }
                }
            }
            Expr::Block(stmts) => {
                let mut last_val = builder.ins().iconst(types::I64, 0);
                for stmt in stmts {
                    if let Some(cur_b) = builder.current_block() {
                        if is_block_terminated(builder, cur_b) {
                            break;
                        }
                    }
                    match stmt {
                        Statement::Let { name, value, .. } => {
                            let val = self.compile_expr(builder, value, vars, defers, loop_ctx)?;
                            let var = builder.declare_var(types::I64);
                            builder.def_var(var, val);
                            vars.insert(name.clone(), (var, types::I64));

                            if let Some(&data_id) = self.globals.get(name) {
                                let local_data =
                                    self.module.declare_data_in_func(data_id, builder.func);
                                let addr = builder.ins().symbol_value(types::I64, local_data);
                                builder.ins().store(MemFlagsData::trusted(), val, addr, 0);
                            }
                        }
                        Statement::Assign { target, value } => {
                            let val = self.compile_expr(builder, value, vars, defers, loop_ctx)?;
                            match target {
                                Expr::Identifier(name) => {
                                    if let Some((var, _)) = vars.get(name) {
                                        builder.def_var(*var, val);
                                    }
                                    if let Some(&data_id) = self.globals.get(name) {
                                        let local_data =
                                            self.module.declare_data_in_func(data_id, builder.func);
                                        let addr =
                                            builder.ins().symbol_value(types::I64, local_data);
                                        builder.ins().store(MemFlagsData::trusted(), val, addr, 0);
                                    }
                                }
                                Expr::IndexAccess { object, index } => {
                                    let obj_val =
                                        self.compile_expr(builder, object, vars, defers, loop_ctx)?;
                                    let idx_val =
                                        self.compile_expr(builder, index, vars, defers, loop_ctx)?;

                                    let zero = builder.ins().iconst(types::I64, 0);
                                    let not_null = builder.ins().icmp(
                                        cranelift_codegen::ir::condcodes::IntCC::NotEqual,
                                        obj_val,
                                        zero,
                                    );
                                    let check_block = builder.create_block();
                                    let slow_block = builder.create_block();
                                    let fast_block = builder.create_block();
                                    let merge_block = builder.create_block();

                                    builder
                                        .ins()
                                        .brif(not_null, check_block, &[], slow_block, &[]);

                                    builder.switch_to_block(check_block);
                                    builder.seal_block(check_block);
                                    let len_val = builder.ins().load(
                                        types::I64,
                                        MemFlagsData::trusted(),
                                        obj_val,
                                        16,
                                    );
                                    let in_bounds = builder.ins().icmp(
                                        cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThan,
                                        idx_val,
                                        len_val,
                                    );
                                    builder
                                        .ins()
                                        .brif(in_bounds, fast_block, &[], slow_block, &[]);

                                    builder.switch_to_block(fast_block);
                                    builder.seal_block(fast_block);
                                    let items_ptr = builder.ins().load(
                                        types::I64,
                                        MemFlagsData::trusted(),
                                        obj_val,
                                        8,
                                    );
                                    let offset = builder.ins().ishl_imm_u(idx_val, 3);
                                    let addr = builder.ins().iadd(items_ptr, offset);
                                    builder.ins().store(MemFlagsData::trusted(), val, addr, 0);
                                    builder.ins().jump(merge_block, &[]);

                                    builder.switch_to_block(slow_block);
                                    builder.seal_block(slow_block);
                                    let local_set = self
                                        .module
                                        .declare_func_in_func(self.list_set_id, builder.func);
                                    builder.ins().call(local_set, &[obj_val, idx_val, val]);
                                    builder.ins().jump(merge_block, &[]);

                                    builder.switch_to_block(merge_block);
                                    builder.seal_block(merge_block);
                                }
                                Expr::MemberAccess { object, member } => {
                                    let obj_val =
                                        self.compile_expr(builder, object, vars, defers, loop_ctx)?;
                                    let k_val = self.emit_string_literal(builder, member)?;
                                    let (ptr_val, len_val) =
                                        self.get_string_ptr_and_len(builder, k_val);
                                    let local_set = self
                                        .module
                                        .declare_func_in_func(self.record_set_id, builder.func);
                                    builder
                                        .ins()
                                        .call(local_set, &[obj_val, ptr_val, len_val, val]);
                                }
                                _ => {}
                            }
                        }
                        Statement::Defer(e) => {
                            defers.push((*e.clone(), false));
                        }
                        Statement::ErrDefer(e) => {
                            defers.push((*e.clone(), true));
                        }
                        Statement::Expr(e) => {
                            last_val = self.compile_expr(builder, e, vars, defers, loop_ctx)?;
                        }
                        Statement::Return(Some(e)) => {
                            let ret = self.compile_expr(builder, e, vars, defers, loop_ctx)?;
                            self.compile_defers(builder, defers, vars, loop_ctx, ret)?;
                            builder.ins().return_(&[ret]);
                        }
                        Statement::Return(None) => {
                            let zero = builder.ins().iconst(types::I64, 0);
                            self.compile_defers(builder, defers, vars, loop_ctx, zero)?;
                            builder.ins().return_(&[zero]);
                        }
                        _ => {}
                    }
                }
                Ok(last_val)
            }
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_val = self.compile_expr(builder, condition, vars, defers, loop_ctx)?;
                let then_block = builder.create_block();
                let else_block = builder.create_block();
                let merge_block = builder.create_block();

                let res_var = builder.declare_var(types::I64);

                let zero = builder.ins().iconst(types::I64, 0);
                let is_true = builder.ins().icmp(
                    cranelift_codegen::ir::condcodes::IntCC::NotEqual,
                    cond_val,
                    zero,
                );
                builder
                    .ins()
                    .brif(is_true, then_block, &[], else_block, &[]);

                // Then branch
                builder.switch_to_block(then_block);
                builder.seal_block(then_block);
                let then_res = self.compile_expr(builder, then_branch, vars, defers, loop_ctx)?;
                if let Some(cur) = builder.current_block() {
                    if !is_block_terminated(builder, cur) {
                        builder.def_var(res_var, then_res);
                        builder.ins().jump(merge_block, &[]);
                    }
                }

                // Else branch
                builder.switch_to_block(else_block);
                builder.seal_block(else_block);
                let else_res = if let Some(else_b) = else_branch {
                    self.compile_expr(builder, else_b, vars, defers, loop_ctx)?
                } else {
                    builder.ins().iconst(types::I64, 0)
                };
                if let Some(cur) = builder.current_block() {
                    if !is_block_terminated(builder, cur) {
                        builder.def_var(res_var, else_res);
                        builder.ins().jump(merge_block, &[]);
                    }
                }

                // Merge block
                builder.switch_to_block(merge_block);
                builder.seal_block(merge_block);
                Ok(builder.use_var(res_var))
            }
            Expr::While {
                condition, body, ..
            } => {
                let header_block = builder.create_block();
                let body_block = builder.create_block();
                let exit_block = builder.create_block();

                builder.ins().jump(header_block, &[]);
                builder.switch_to_block(header_block);

                let cond_val = self.compile_expr(builder, condition, vars, defers, loop_ctx)?;
                let zero = builder.ins().iconst(types::I64, 0);
                let is_true = builder.ins().icmp(
                    cranelift_codegen::ir::condcodes::IntCC::NotEqual,
                    cond_val,
                    zero,
                );
                builder
                    .ins()
                    .brif(is_true, body_block, &[], exit_block, &[]);

                builder.switch_to_block(body_block);
                builder.seal_block(body_block);
                self.compile_expr(
                    builder,
                    body,
                    vars,
                    defers,
                    Some((header_block, exit_block)),
                )?;
                let should_loop = if let Some(cur) = builder.current_block() {
                    !is_block_terminated(builder, cur)
                } else {
                    false
                };
                if should_loop && !builder.is_unreachable() {
                    builder.ins().jump(header_block, &[]);
                }

                builder.seal_block(header_block);
                builder.switch_to_block(exit_block);
                builder.seal_block(exit_block);

                Ok(zero)
            }
            Expr::Break(_) => {
                let zero = builder.ins().iconst(types::I64, 0);
                if let Some((_, exit_b)) = loop_ctx {
                    builder.ins().jump(exit_b, &[]);
                }
                Ok(zero)
            }
            Expr::Continue(_) => {
                let zero = builder.ins().iconst(types::I64, 0);
                if let Some((hdr_b, _)) = loop_ctx {
                    builder.ins().jump(hdr_b, &[]);
                }
                Ok(zero)
            }
            Expr::FunctionCall { callee, args } => {
                self.compile_function_call(builder, callee, args, vars, defers, loop_ctx)
            }
            Expr::Lambda { params, body, .. } => self.emit_lambda(params, body).map(|func_id| {
                let local_func = self.module.declare_func_in_func(func_id, builder.func);
                builder.ins().func_addr(types::I64, local_func)
            }),
            Expr::Spawn(fiber_body) => {
                let fiber_name = format!("__aura_fiber_{}", self.fiber_counter);
                self.fiber_counter += 1;

                let fiber_func_id = self.emit_fiber_function(&fiber_name, fiber_body)?;
                let local_fiber_func = self
                    .module
                    .declare_func_in_func(fiber_func_id, builder.func);
                let fn_ptr = builder.ins().func_addr(types::I64, local_fiber_func);
                let null_arg = builder.ins().iconst(types::I64, 0);

                let local_spawn = self
                    .module
                    .declare_func_in_func(self.fiber_spawn_id, builder.func);
                let call = builder.ins().call(local_spawn, &[fn_ptr, null_arg]);
                Ok(builder.inst_results(call)[0])
            }
            Expr::ChanSend { channel, value } => {
                let ch_val = self.compile_expr(builder, channel, vars, defers, loop_ctx)?;
                let v_val = self.compile_expr(builder, value, vars, defers, loop_ctx)?;
                let local_send = self
                    .module
                    .declare_func_in_func(self.chan_send_id, builder.func);
                builder.ins().call(local_send, &[ch_val, v_val]);
                Ok(builder.ins().iconst(types::I64, 0))
            }
            Expr::ChanRecv(channel) => {
                let ch_val = self.compile_expr(builder, channel, vars, defers, loop_ctx)?;
                let local_recv = self
                    .module
                    .declare_func_in_func(self.chan_recv_id, builder.func);
                let call = builder.ins().call(local_recv, &[ch_val]);
                let raw_val = builder.inst_results(call)[0];

                // Wrap into Option: if raw_val != 0 -> Some(raw_val) else None
                let local_some = self
                    .module
                    .declare_func_in_func(self.option_some_id, builder.func);
                let call_some = builder.ins().call(local_some, &[raw_val]);
                Ok(builder.inst_results(call_some)[0])
            }
            Expr::ConstructorCall { name, args } => match name.as_str() {
                "Some" => {
                    let arg_val = if !args.is_empty() {
                        self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?
                    } else {
                        builder.ins().iconst(types::I64, 0)
                    };
                    let local_some = self
                        .module
                        .declare_func_in_func(self.option_some_id, builder.func);
                    let call = builder.ins().call(local_some, &[arg_val]);
                    Ok(builder.inst_results(call)[0])
                }
                "None" => {
                    let local_none = self
                        .module
                        .declare_func_in_func(self.option_none_id, builder.func);
                    let call = builder.ins().call(local_none, &[]);
                    Ok(builder.inst_results(call)[0])
                }
                "Ok" => {
                    let arg_val = if !args.is_empty() {
                        self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?
                    } else {
                        builder.ins().iconst(types::I64, 0)
                    };
                    let local_ok = self
                        .module
                        .declare_func_in_func(self.result_ok_id, builder.func);
                    let call = builder.ins().call(local_ok, &[arg_val]);
                    Ok(builder.inst_results(call)[0])
                }
                "Err" => {
                    let arg_val = if !args.is_empty() {
                        self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?
                    } else {
                        builder.ins().iconst(types::I64, 0)
                    };
                    let local_err = self
                        .module
                        .declare_func_in_func(self.result_err_id, builder.func);
                    let call = builder.ins().call(local_err, &[arg_val]);
                    Ok(builder.inst_results(call)[0])
                }
                _ => Ok(builder.ins().iconst(types::I64, 0)),
            },
            Expr::Match { subject, arms } => {
                let sub_val = self.compile_expr(builder, subject, vars, defers, loop_ctx)?;
                let merge_block = builder.create_block();
                let res_var = builder.declare_var(types::I64);

                let local_tag = self
                    .module
                    .declare_func_in_func(self.variant_tag_id, builder.func);
                let call_tag = builder.ins().call(local_tag, &[sub_val]);
                let tag_val = builder.inst_results(call_tag)[0];

                let mut current_test_block = builder.current_block().unwrap();

                for (idx, arm) in arms.iter().enumerate() {
                    let arm_exec_block = builder.create_block();
                    let next_test_block = builder.create_block();

                    match &arm.pattern {
                        Pattern::Constructor { name, patterns } => {
                            let expected_tag = match name.as_str() {
                                "Some" | "Ok" => 1,
                                "Err" => 2,
                                "None" => 0,
                                _ => 1,
                            };
                            let tag_const = builder.ins().iconst(types::I64, expected_tag);
                            let matches = builder.ins().icmp(
                                cranelift_codegen::ir::condcodes::IntCC::Equal,
                                tag_val,
                                tag_const,
                            );
                            builder
                                .ins()
                                .brif(matches, arm_exec_block, &[], next_test_block, &[]);

                            // Arm body execution
                            builder.switch_to_block(arm_exec_block);
                            builder.seal_block(arm_exec_block);

                            if !patterns.is_empty() {
                                if let Pattern::Variable(v_name) = &patterns[0] {
                                    let local_val_fn = self
                                        .module
                                        .declare_func_in_func(self.variant_val_id, builder.func);
                                    let call_val = builder.ins().call(local_val_fn, &[sub_val]);
                                    let inner_val = builder.inst_results(call_val)[0];

                                    let var = builder.declare_var(types::I64);
                                    builder.def_var(var, inner_val);
                                    vars.insert(v_name.clone(), (var, types::I64));
                                    if let Some(&data_id) = self.globals.get(v_name) {
                                        let local_data =
                                            self.module.declare_data_in_func(data_id, builder.func);
                                        let addr =
                                            builder.ins().symbol_value(types::I64, local_data);
                                        builder.ins().store(
                                            MemFlagsData::trusted(),
                                            inner_val,
                                            addr,
                                            0,
                                        );
                                    }
                                }
                            }

                            let body_res =
                                self.compile_expr(builder, &arm.body, vars, defers, loop_ctx)?;
                            if !builder.is_unreachable() {
                                builder.def_var(res_var, body_res);
                                builder.ins().jump(merge_block, &[]);
                            }
                        }
                        Pattern::Wildcard => {
                            builder.ins().jump(arm_exec_block, &[]);
                            builder.switch_to_block(arm_exec_block);
                            builder.seal_block(arm_exec_block);

                            let body_res =
                                self.compile_expr(builder, &arm.body, vars, defers, loop_ctx)?;
                            if !builder.is_unreachable() {
                                builder.def_var(res_var, body_res);
                                builder.ins().jump(merge_block, &[]);
                            }
                        }
                        _ => {
                            builder.ins().jump(next_test_block, &[]);
                        }
                    }

                    if idx > 0 {
                        builder.seal_block(current_test_block);
                    }
                    builder.switch_to_block(next_test_block);
                    current_test_block = next_test_block;
                }

                // Fallback on next_test_block
                builder.seal_block(current_test_block);
                let zero = builder.ins().iconst(types::I64, 0);
                builder.def_var(res_var, zero);
                builder.ins().jump(merge_block, &[]);

                builder.switch_to_block(merge_block);
                builder.seal_block(merge_block);
                Ok(builder.use_var(res_var))
            }
            Expr::Try(inner) => {
                let inner_val = self.compile_expr(builder, inner, vars, defers, loop_ctx)?;
                let local_tag = self
                    .module
                    .declare_func_in_func(self.variant_tag_id, builder.func);
                let call_tag = builder.ins().call(local_tag, &[inner_val]);
                let tag_val = builder.inst_results(call_tag)[0];
                let tag_err = builder.ins().iconst(types::I64, 2);
                let is_err = builder.ins().icmp(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    tag_val,
                    tag_err,
                );

                let err_block = builder.create_block();
                let ok_block = builder.create_block();
                builder.ins().brif(is_err, err_block, &[], ok_block, &[]);

                // Error path: unwind all defers (including errdefer) and early-return inner_val
                builder.switch_to_block(err_block);
                builder.seal_block(err_block);
                for (d, _) in defers.iter().rev() {
                    let mut sub_defers = Vec::new();
                    self.compile_expr(builder, d, vars, &mut sub_defers, loop_ctx)?;
                }
                builder.ins().return_(&[inner_val]);

                // Ok path: extract inner value
                builder.switch_to_block(ok_block);
                builder.seal_block(ok_block);
                let local_val_fn = self
                    .module
                    .declare_func_in_func(self.variant_val_id, builder.func);
                let call_val = builder.ins().call(local_val_fn, &[inner_val]);
                Ok(builder.inst_results(call_val)[0])
            }
            Expr::Async(inner) | Expr::Await(inner) => {
                self.compile_expr(builder, inner, vars, defers, loop_ctx)
            }
            _ => Ok(builder.ins().iconst(types::I64, 0)),
        }
    }

    fn compile_function_call(
        &mut self,
        builder: &mut FunctionBuilder,
        callee: &Expr,
        args: &[Expr],
        vars: &mut HashMap<String, (Variable, types::Type)>,
        defers: &mut Vec<(Expr, bool)>,
        loop_ctx: Option<(cranelift_codegen::ir::Block, cranelift_codegen::ir::Block)>,
    ) -> Result<Value, String> {
        // Direct method call: object.method(args)
        if let Expr::MemberAccess { object, member } = callee {
            // Builtin static calls: Mutex.new(), Channel.make(100), os.env(...)
            if let Expr::Identifier(ref obj_name) = **object {
                if obj_name == "Mutex" && member == "new" {
                    let local_fn = self
                        .module
                        .declare_func_in_func(self.mutex_new_id, builder.func);
                    let call = builder.ins().call(local_fn, &[]);
                    return Ok(builder.inst_results(call)[0]);
                }
                if (obj_name == "Channel" && member == "make")
                    || (obj_name == "Channel" && member == "new")
                {
                    let cap = if !args.is_empty() {
                        self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?
                    } else {
                        builder.ins().iconst(types::I64, 0)
                    };
                    let local_fn = self
                        .module
                        .declare_func_in_func(self.chan_new_id, builder.func);
                    let call = builder.ins().call(local_fn, &[cap]);
                    return Ok(builder.inst_results(call)[0]);
                }
                if obj_name == "os" && member == "env" && !args.is_empty() {
                    let str_val = self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?;
                    let (ptr_val, len_val) = self.get_string_ptr_and_len(builder, str_val);

                    let local_fn = self
                        .module
                        .declare_func_in_func(self.os_env_id, builder.func);
                    let call = builder.ins().call(local_fn, &[ptr_val, len_val]);
                    return Ok(builder.inst_results(call)[0]);
                }
                if obj_name == "os" && member == "hostname" {
                    let local_fn = self
                        .module
                        .declare_func_in_func(self.os_hostname_id, builder.func);
                    let call = builder.ins().call(local_fn, &[]);
                    return Ok(builder.inst_results(call)[0]);
                }
                if obj_name == "process" && member == "uptime" {
                    let local_fn = self
                        .module
                        .declare_func_in_func(self.process_uptime_id, builder.func);
                    let call = builder.ins().call(local_fn, &[]);
                    return Ok(builder.inst_results(call)[0]);
                }
                if obj_name == "time" && member == "now" {
                    let local_fn = self
                        .module
                        .declare_func_in_func(self.time_now_id, builder.func);
                    let call = builder.ins().call(local_fn, &[]);
                    return Ok(builder.inst_results(call)[0]);
                }
                if obj_name == "http" {
                    match member.as_str() {
                        "newServeMux" => {
                            let local_fn = self
                                .module
                                .declare_func_in_func(self.http_new_serve_mux_id, builder.func);
                            let call = builder.ins().call(local_fn, &[]);
                            return Ok(builder.inst_results(call)[0]);
                        }
                        "json" if args.len() >= 3 => {
                            let res_val =
                                self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?;
                            let status_val =
                                self.compile_expr(builder, &args[1], vars, defers, loop_ctx)?;
                            let body_val =
                                self.compile_expr(builder, &args[2], vars, defers, loop_ctx)?;
                            let local_fn = self
                                .module
                                .declare_func_in_func(self.http_json_id, builder.func);
                            builder
                                .ins()
                                .call(local_fn, &[res_val, status_val, body_val]);
                            return Ok(builder.ins().iconst(types::I64, 0));
                        }
                        "error" if args.len() >= 3 => {
                            let res_val =
                                self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?;
                            let msg_val =
                                self.compile_expr(builder, &args[1], vars, defers, loop_ctx)?;
                            let status_val =
                                self.compile_expr(builder, &args[2], vars, defers, loop_ctx)?;

                            let (ptr_val, len_val) = self.get_string_ptr_and_len(builder, msg_val);

                            let local_fn = self
                                .module
                                .declare_func_in_func(self.http_error_id, builder.func);
                            builder
                                .ins()
                                .call(local_fn, &[res_val, ptr_val, len_val, status_val]);
                            return Ok(builder.ins().iconst(types::I64, 0));
                        }
                        "pathValue" if args.len() >= 2 => {
                            let req_val =
                                self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?;
                            let key_val =
                                self.compile_expr(builder, &args[1], vars, defers, loop_ctx)?;
                            let (ptr_val, len_val) = self.get_string_ptr_and_len(builder, key_val);

                            let local_fn = self
                                .module
                                .declare_func_in_func(self.http_path_value_id, builder.func);
                            let call = builder.ins().call(local_fn, &[req_val, ptr_val, len_val]);
                            return Ok(builder.inst_results(call)[0]);
                        }
                        "query" if args.len() >= 2 => {
                            let req_val =
                                self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?;
                            let key_val =
                                self.compile_expr(builder, &args[1], vars, defers, loop_ctx)?;
                            let (ptr_val, len_val) = self.get_string_ptr_and_len(builder, key_val);

                            let local_fn = self
                                .module
                                .declare_func_in_func(self.http_query_id, builder.func);
                            let call = builder.ins().call(local_fn, &[req_val, ptr_val, len_val]);
                            return Ok(builder.inst_results(call)[0]);
                        }
                        "parseJson" if !args.is_empty() => {
                            let req_val =
                                self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?;
                            let local_fn = self
                                .module
                                .declare_func_in_func(self.http_parse_json_id, builder.func);
                            let call = builder.ins().call(local_fn, &[req_val]);
                            return Ok(builder.inst_results(call)[0]);
                        }
                        _ => {}
                    }
                }
            }

            // Instance method calls: target.lock(), target.push(item), etc.
            let target_val = self.compile_expr(builder, object, vars, defers, loop_ctx)?;

            match member.as_str() {
                "lock" => {
                    let local_fn = self
                        .module
                        .declare_func_in_func(self.mutex_lock_id, builder.func);
                    builder.ins().call(local_fn, &[target_val]);
                    return Ok(builder.ins().iconst(types::I64, 0));
                }
                "unlock" => {
                    let local_fn = self
                        .module
                        .declare_func_in_func(self.mutex_unlock_id, builder.func);
                    builder.ins().call(local_fn, &[target_val]);
                    return Ok(builder.ins().iconst(types::I64, 0));
                }
                "push" if !args.is_empty() => {
                    let item_val = self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?;

                    let zero = builder.ins().iconst(types::I64, 0);
                    let not_null = builder.ins().icmp(
                        cranelift_codegen::ir::condcodes::IntCC::NotEqual,
                        target_val,
                        zero,
                    );
                    let check_block = builder.create_block();
                    let slow_block = builder.create_block();
                    let fast_block = builder.create_block();
                    let merge_block = builder.create_block();

                    builder
                        .ins()
                        .brif(not_null, check_block, &[], slow_block, &[]);

                    builder.switch_to_block(check_block);
                    builder.seal_block(check_block);
                    let len_val =
                        builder
                            .ins()
                            .load(types::I64, MemFlagsData::trusted(), target_val, 16);
                    let cap_val =
                        builder
                            .ins()
                            .load(types::I64, MemFlagsData::trusted(), target_val, 24);
                    let has_cap = builder.ins().icmp(
                        cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThan,
                        len_val,
                        cap_val,
                    );
                    builder
                        .ins()
                        .brif(has_cap, fast_block, &[], slow_block, &[]);

                    builder.switch_to_block(fast_block);
                    builder.seal_block(fast_block);
                    let items_ptr =
                        builder
                            .ins()
                            .load(types::I64, MemFlagsData::trusted(), target_val, 8);
                    let offset = builder.ins().ishl_imm_u(len_val, 3);
                    let addr = builder.ins().iadd(items_ptr, offset);
                    builder
                        .ins()
                        .store(MemFlagsData::trusted(), item_val, addr, 0);
                    let one = builder.ins().iconst(types::I64, 1);
                    let new_len = builder.ins().iadd(len_val, one);
                    builder
                        .ins()
                        .store(MemFlagsData::trusted(), new_len, target_val, 16);
                    builder.ins().jump(merge_block, &[]);

                    builder.switch_to_block(slow_block);
                    builder.seal_block(slow_block);
                    let local_fn = self
                        .module
                        .declare_func_in_func(self.list_push_id, builder.func);
                    builder.ins().call(local_fn, &[target_val, item_val]);
                    builder.ins().jump(merge_block, &[]);

                    builder.switch_to_block(merge_block);
                    builder.seal_block(merge_block);
                    return Ok(builder.ins().iconst(types::I64, 0));
                }
                "toLowerCase" => {
                    let local_fn = self
                        .module
                        .declare_func_in_func(self.string_to_lower_id, builder.func);
                    let call = builder.ins().call(local_fn, &[target_val]);
                    return Ok(builder.inst_results(call)[0]);
                }
                "includes" if !args.is_empty() => {
                    let sub_val = self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?;
                    let local_fn = self
                        .module
                        .declare_func_in_func(self.string_contains_id, builder.func);
                    let call = builder.ins().call(local_fn, &[target_val, sub_val]);
                    return Ok(builder.inst_results(call)[0]);
                }
                "setHeader" if args.len() >= 2 => {
                    let k_val = self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?;
                    let v_val = self.compile_expr(builder, &args[1], vars, defers, loop_ctx)?;

                    let (ptr_k, len_k) = self.get_string_ptr_and_len(builder, k_val);
                    let (ptr_v, len_v) = self.get_string_ptr_and_len(builder, v_val);

                    let local_fn = self
                        .module
                        .declare_func_in_func(self.http_set_header_id, builder.func);
                    builder
                        .ins()
                        .call(local_fn, &[target_val, ptr_k, len_k, ptr_v, len_v]);
                    return Ok(builder.ins().iconst(types::I64, 0));
                }
                "use" if !args.is_empty() => {
                    let mw_fn = self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?;
                    let local_fn = self
                        .module
                        .declare_func_in_func(self.http_mux_use_id, builder.func);
                    builder.ins().call(local_fn, &[target_val, mw_fn]);
                    return Ok(builder.ins().iconst(types::I64, 0));
                }
                "get" | "post" | "put" | "delete" if args.len() >= 2 => {
                    let method = member.to_uppercase();
                    let m_val = self.emit_string_literal(builder, &method)?;
                    let (m_ptr, m_len) = self.get_string_ptr_and_len(builder, m_val);

                    let p_val = self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?;
                    let (p_ptr, p_len) = self.get_string_ptr_and_len(builder, p_val);

                    let handler_fn =
                        self.compile_expr(builder, &args[1], vars, defers, loop_ctx)?;

                    let local_fn = self
                        .module
                        .declare_func_in_func(self.http_mux_handle_id, builder.func);
                    builder.ins().call(
                        local_fn,
                        &[target_val, m_ptr, m_len, p_ptr, p_len, handler_fn],
                    );
                    return Ok(builder.ins().iconst(types::I64, 0));
                }
                "listenAndServe" if !args.is_empty() => {
                    let addr_val = self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?;
                    let (addr_ptr, addr_len) = self.get_string_ptr_and_len(builder, addr_val);

                    let local_fn = self
                        .module
                        .declare_func_in_func(self.http_listen_and_serve_id, builder.func);
                    builder
                        .ins()
                        .call(local_fn, &[target_val, addr_ptr, addr_len]);
                    return Ok(builder.ins().iconst(types::I64, 0));
                }
                "filter" if !args.is_empty() => {
                    let pred_fn = self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?;
                    let local_fn = self
                        .module
                        .declare_func_in_func(self.list_filter_id, builder.func);
                    let call = builder.ins().call(local_fn, &[target_val, pred_fn]);
                    return Ok(builder.inst_results(call)[0]);
                }
                "map" if !args.is_empty() => {
                    let map_fn = self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?;
                    let local_fn = self
                        .module
                        .declare_func_in_func(self.list_map_id, builder.func);
                    let call = builder.ins().call(local_fn, &[target_val, map_fn]);
                    return Ok(builder.inst_results(call)[0]);
                }
                "slice" => {
                    let s_val = if !args.is_empty() {
                        self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?
                    } else {
                        builder.ins().iconst(types::I64, 0)
                    };
                    let e_val = if args.len() >= 2 {
                        self.compile_expr(builder, &args[1], vars, defers, loop_ctx)?
                    } else {
                        builder.ins().iconst(types::I64, -1)
                    };
                    let local_fn = self
                        .module
                        .declare_func_in_func(self.list_slice_id, builder.func);
                    let call = builder.ins().call(local_fn, &[target_val, s_val, e_val]);
                    return Ok(builder.inst_results(call)[0]);
                }
                _ => {}
            }
        }

        // Direct identifier calls: println, sleep, parseFloat, user functions
        if let Expr::Identifier(ref name) = *callee {
            if (name == "len" || name == "length") && !args.is_empty() {
                let arg_val = self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?;
                let local_fn = self
                    .module
                    .declare_func_in_func(self.list_len_id, builder.func);
                let call = builder.ins().call(local_fn, &[arg_val]);
                return Ok(builder.inst_results(call)[0]);
            }

            if (name == "cap" || name == "capacity") && !args.is_empty() {
                let arg_val = self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?;
                let local_fn = self
                    .module
                    .declare_func_in_func(self.list_cap_id, builder.func);
                let call = builder.ins().call(local_fn, &[arg_val]);
                return Ok(builder.inst_results(call)[0]);
            }

            if name == "append" && args.len() >= 2 {
                let target_val = self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?;
                let item_val = self.compile_expr(builder, &args[1], vars, defers, loop_ctx)?;
                let local_fn = self
                    .module
                    .declare_func_in_func(self.list_push_id, builder.func);
                builder.ins().call(local_fn, &[target_val, item_val]);
                return Ok(target_val);
            }
            if name == "println" {
                let arg_val = if !args.is_empty() {
                    self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?
                } else {
                    builder.ins().iconst(types::I64, 0)
                };
                let local_fn = self
                    .module
                    .declare_func_in_func(self.println_any_id, builder.func);
                builder.ins().call(local_fn, &[arg_val]);
                return Ok(builder.ins().iconst(types::I64, 0));
            }

            if name == "yield" {
                let local_fn = self
                    .module
                    .declare_func_in_func(self.fiber_yield_id, builder.func);
                builder.ins().call(local_fn, &[]);
                return Ok(builder.ins().iconst(types::I64, 0));
            }

            if name == "sleep" && !args.is_empty() {
                let ms_val = self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?;
                let local_fn = self
                    .module
                    .declare_func_in_func(self.sleep_ms_id, builder.func);
                builder.ins().call(local_fn, &[ms_val]);
                return Ok(builder.ins().iconst(types::I64, 0));
            }

            if name == "parseFloat" && !args.is_empty() {
                let str_val = self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?;
                let local_fn = self
                    .module
                    .declare_func_in_func(self.parse_float_id, builder.func);
                let call = builder.ins().call(local_fn, &[str_val]);
                return Ok(builder.inst_results(call)[0]);
            }

            if name == "Some" {
                let arg_val = if !args.is_empty() {
                    self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?
                } else {
                    builder.ins().iconst(types::I64, 0)
                };
                let local_some = self
                    .module
                    .declare_func_in_func(self.option_some_id, builder.func);
                let call = builder.ins().call(local_some, &[arg_val]);
                return Ok(builder.inst_results(call)[0]);
            }

            if name == "None" {
                let local_none = self
                    .module
                    .declare_func_in_func(self.option_none_id, builder.func);
                let call = builder.ins().call(local_none, &[]);
                return Ok(builder.inst_results(call)[0]);
            }

            if name == "Ok" {
                let arg_val = if !args.is_empty() {
                    self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?
                } else {
                    builder.ins().iconst(types::I64, 0)
                };
                let local_ok = self
                    .module
                    .declare_func_in_func(self.result_ok_id, builder.func);
                let call = builder.ins().call(local_ok, &[arg_val]);
                return Ok(builder.inst_results(call)[0]);
            }

            if name == "Err" {
                let arg_val = if !args.is_empty() {
                    self.compile_expr(builder, &args[0], vars, defers, loop_ctx)?
                } else {
                    builder.ins().iconst(types::I64, 0)
                };
                let local_err = self
                    .module
                    .declare_func_in_func(self.result_err_id, builder.func);
                let call = builder.ins().call(local_err, &[arg_val]);
                return Ok(builder.inst_results(call)[0]);
            }

            if let Some(&func_id) = self.func_ids.get(name) {
                let mut compiled_args = Vec::new();
                for a in args {
                    compiled_args.push(self.compile_expr(builder, a, vars, defers, loop_ctx)?);
                }
                let local_func = self.module.declare_func_in_func(func_id, builder.func);
                let call_inst = builder.ins().call(local_func, &compiled_args);
                return Ok(builder.inst_results(call_inst)[0]);
            }
        }

        // Indirect function call through variable pointer
        let fn_val = self.compile_expr(builder, callee, vars, defers, loop_ctx)?;
        let mut compiled_args = Vec::new();
        for a in args {
            compiled_args.push(self.compile_expr(builder, a, vars, defers, loop_ctx)?);
        }

        let target_config = self.module.target_config();
        let mut sig = Signature::new(target_config.default_call_conv);
        for _ in 0..compiled_args.len() {
            sig.params.push(AbiParam::new(types::I64));
        }
        sig.returns.push(AbiParam::new(types::I64));
        let sig_ref = builder.import_signature(sig);

        let call_inst = builder.ins().call_indirect(sig_ref, fn_val, &compiled_args);
        let res = builder.inst_results(call_inst);
        if res.is_empty() {
            Ok(builder.ins().iconst(types::I64, 0))
        } else {
            Ok(res[0])
        }
    }

    fn emit_lambda(&mut self, params: &[Param], body: &Expr) -> Result<FuncId, String> {
        let name = format!("__aura_lambda_{}", self.lambda_counter);
        self.lambda_counter += 1;

        let call_conv = self.module.target_config().default_call_conv;
        let mut sig = Signature::new(call_conv);
        for _ in params {
            sig.params.push(AbiParam::new(types::I64));
        }
        sig.returns.push(AbiParam::new(types::I64));

        let func_id = self
            .module
            .declare_function(&name, Linkage::Local, &sig)
            .map_err(|e| format!("Error declaring lambda '{}': {}", name, e))?;

        let mut ctx = self.module.make_context();
        ctx.func.signature = sig;

        let mut fn_builder_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut fn_builder_ctx);
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        let mut vars = HashMap::new();
        let mut defers = Vec::new();

        for (i, p) in params.iter().enumerate() {
            let var = builder.declare_var(types::I64);
            let val = builder.block_params(entry_block)[i];
            builder.def_var(var, val);
            vars.insert(p.name.clone(), (var, types::I64));
            if let Some(&data_id) = self.globals.get(&p.name) {
                let local_data = self.module.declare_data_in_func(data_id, builder.func);
                let addr = builder.ins().symbol_value(types::I64, local_data);
                builder.ins().store(MemFlagsData::trusted(), val, addr, 0);
            }
        }

        let ret_val = self.compile_expr(&mut builder, body, &mut vars, &mut defers, None)?;

        self.compile_defers(&mut builder, &defers, &mut vars, None, ret_val)?;

        let should_return = if let Some(cur) = builder.current_block() {
            !is_block_terminated(&builder, cur)
        } else {
            false
        };
        if should_return && !builder.is_unreachable() {
            builder.ins().return_(&[ret_val]);
        }
        builder.finalize(self.module.target_config());

        self.module
            .define_function(func_id, &mut ctx)
            .map_err(|e| format!("Error defining lambda '{}': {}", name, e))?;

        Ok(func_id)
    }

    fn emit_fiber_function(&mut self, name: &str, body: &Expr) -> Result<FuncId, String> {
        let call_conv = self.module.target_config().default_call_conv;
        let mut sig = Signature::new(call_conv);
        sig.params.push(AbiParam::new(types::I64)); // arg: *mut ()

        let func_id = self
            .module
            .declare_function(name, Linkage::Local, &sig)
            .map_err(|e| format!("Error declaring fiber function '{}': {}", name, e))?;

        let mut ctx = self.module.make_context();
        ctx.func.signature = sig;

        let mut fn_builder_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut fn_builder_ctx);
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        let mut vars = HashMap::new();
        let mut defers = Vec::new();
        self.compile_expr(&mut builder, body, &mut vars, &mut defers, None)?;

        let zero = builder.ins().iconst(types::I64, 0);
        self.compile_defers(&mut builder, &defers, &mut vars, None, zero)?;

        let should_return = if let Some(cur) = builder.current_block() {
            !is_block_terminated(&builder, cur)
        } else {
            false
        };
        if should_return && !builder.is_unreachable() {
            builder.ins().return_(&[]);
        }
        builder.finalize(self.module.target_config());

        self.module
            .define_function(func_id, &mut ctx)
            .map_err(|e| format!("Error defining fiber function '{}': {}", name, e))?;

        Ok(func_id)
    }

    fn emit_c_main_entrypoint(&mut self) -> Result<(), String> {
        let call_conv = self.module.target_config().default_call_conv;
        let mut sig = Signature::new(call_conv);
        sig.returns.push(AbiParam::new(types::I64)); // int main()

        let main_id = self
            .module
            .declare_function("main", Linkage::Export, &sig)
            .map_err(|e| format!("Error declaring 'main': {}", e))?;

        let mut ctx = self.module.make_context();
        ctx.func.signature = sig;

        let mut fn_builder_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut fn_builder_ctx);
        let entry_block = builder.create_block();
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        let user_main_id = self.func_ids["main"];
        let local_user_main = self.module.declare_func_in_func(user_main_id, builder.func);
        let user_main_ptr = builder.ins().func_addr(types::I64, local_user_main);

        let local_rt_start = self
            .module
            .declare_func_in_func(self.rt_start_id, builder.func);
        builder.ins().call(local_rt_start, &[user_main_ptr]);

        let zero = builder.ins().iconst(types::I64, 0);
        builder.ins().return_(&[zero]);
        builder.finalize(self.module.target_config());

        self.module
            .define_function(main_id, &mut ctx)
            .map_err(|e| format!("Error defining 'main': {}", e))?;

        Ok(())
    }
}

/// Builds a native standalone binary executable using Cranelift and the Aura Runtime.
pub fn build_native_binary(source: &str, out_path: &Path) -> Result<(), String> {
    build_native_binary_with_base_path(source, out_path, None, None)
}

/// Builds a native standalone binary executable with optional cross-compilation target.
pub fn build_native_binary_with_target(
    source: &str,
    out_path: &Path,
    target: Option<&str>,
) -> Result<(), String> {
    build_native_binary_with_options(source, out_path, None, target, false)
}

/// Builds a native standalone binary executable with optional base path and cross-compilation target.
pub fn build_native_binary_with_base_path(
    source: &str,
    out_path: &Path,
    base_path: Option<&Path>,
    target: Option<&str>,
) -> Result<(), String> {
    build_native_binary_with_options(source, out_path, base_path, target, false)
}

/// Builds a native standalone binary executable with full optimization options (SIMD, LTO dead-stripping).
pub fn build_native_binary_with_options(
    source: &str,
    out_path: &Path,
    base_path: Option<&Path>,
    target: Option<&str>,
    release: bool,
) -> Result<(), String> {
    let mut lexer = crate::lexer::Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = crate::parser::Parser::new(tokens);
    let module = parser.parse_module()?;
    let bundled = crate::bundle_module(&module, base_path)?;

    let backend = CraneliftBackend::with_options(target, release)?;
    let object_bytes = backend.compile_module(&bundled)?;

    let parent_dir = out_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = out_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "app".to_string());

    let tmp_obj_path = parent_dir.join(format!("{}.__aura_tmp.o", stem));
    fs::write(&tmp_obj_path, object_bytes).map_err(|e| {
        format!(
            "Failed to write temp object file '{}': {}",
            tmp_obj_path.display(),
            e
        )
    })?;

    // Locate or build libaura_runtime.a
    let runtime_lib = find_or_build_runtime_lib()?;

    let mut link_cmd = Command::new("cc");
    #[cfg(target_os = "macos")]
    {
        if Path::new("/Library/Developer/CommandLineTools").exists()
            && std::env::var_os("DEVELOPER_DIR").is_none()
        {
            link_cmd.env("DEVELOPER_DIR", "/Library/Developer/CommandLineTools");
        }
        if release {
            link_cmd.arg("-Wl,-dead_strip");
        }
    }
    #[cfg(target_os = "linux")]
    {
        if release {
            link_cmd.arg("-Wl,--gc-sections");
        }
    }
    let link_status = link_cmd
        .arg(&tmp_obj_path)
        .arg(&runtime_lib)
        .arg("-lpthread")
        .arg("-o")
        .arg(out_path)
        .status();

    let _ = fs::remove_file(&tmp_obj_path);

    match link_status {
        Ok(status) if status.success() => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = fs::metadata(out_path) {
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o755);
                    let _ = fs::set_permissions(out_path, perms);
                }
            }
            Ok(())
        }
        Ok(status) => Err(format!("Linker failed with status {}", status)),
        Err(e) => Err(format!("Failed to invoke linker 'cc': {}", e)),
    }
}

fn find_or_build_runtime_lib() -> Result<PathBuf, String> {
    let candidates = [
        PathBuf::from("target/release/libaura_runtime.a"),
        PathBuf::from("target/debug/libaura_runtime.a"),
        PathBuf::from("../target/release/libaura_runtime.a"),
        PathBuf::from("../target/debug/libaura_runtime.a"),
    ];

    for c in &candidates {
        if c.exists() {
            return Ok(c.canonicalize().unwrap_or(c.clone()));
        }
    }

    // Attempt to build it in release mode
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--package")
        .arg("aura-runtime")
        .status();

    if let Ok(s) = status {
        if s.success() {
            for c in &candidates {
                if c.exists() {
                    return Ok(c.canonicalize().unwrap_or(c.clone()));
                }
            }
        }
    }

    Err("Could not find or build libaura_runtime.a".to_string())
}
