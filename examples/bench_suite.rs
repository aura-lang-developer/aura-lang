use aura_lang::codegen::CodeGen;
use aura_lang::compile;
use aura_lang::lexer::Lexer;
use aura_lang::parser::Parser;
use aura_lang::typechecker::TypeChecker;
use std::fs;
use std::time::Instant;

fn bench_stages(name: &str, source: &str) {
    println!("\n------------------------------------------------------------");
    println!("  Análisis de Fases Internas del Compilador: {}", name);
    println!(
        "  Líneas de código: {} | Tamaño: {} bytes",
        source.lines().count(),
        source.len()
    );
    println!("------------------------------------------------------------");

    let iterations = 1000;

    // 1. Lexer
    let start = Instant::now();
    for _ in 0..iterations {
        let mut lexer = Lexer::new(source);
        let _ = lexer.tokenize().unwrap();
    }
    let lex_dur = start.elapsed() / iterations;

    // Tokens for parser
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();

    // 2. Parser
    let start = Instant::now();
    for _ in 0..iterations {
        let mut parser = Parser::new(tokens.clone());
        let _ = parser.parse_module().unwrap();
    }
    let parse_dur = start.elapsed() / iterations;

    let mut parser = Parser::new(tokens.clone());
    let module = parser.parse_module().unwrap();

    // 3. Typechecker
    let start = Instant::now();
    for _ in 0..iterations {
        let mut tc = TypeChecker::new();
        let _ = tc.check_module(&module).unwrap();
    }
    let tc_dur = start.elapsed() / iterations;

    // 4. Codegen
    let start = Instant::now();
    for _ in 0..iterations {
        let mut cg = CodeGen::new();
        let _ = cg.generate(&module);
    }
    let cg_dur = start.elapsed() / iterations;

    // Total End-to-End Core
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = compile(source, &[]).unwrap();
    }
    let total_dur = start.elapsed() / iterations;

    let total_us = total_dur.as_micros() as f64;
    println!(
        "  1. Lexer (Tokenización):       {:>8.2} µs ({:>5.1}%)",
        lex_dur.as_micros() as f64,
        (lex_dur.as_micros() as f64 / total_us) * 100.0
    );
    println!(
        "  2. Parser (Pratt / AST):       {:>8.2} µs ({:>5.1}%)",
        parse_dur.as_micros() as f64,
        (parse_dur.as_micros() as f64 / total_us) * 100.0
    );
    println!(
        "  3. Typechecker (Inferencia):   {:>8.2} µs ({:>5.1}%)",
        tc_dur.as_micros() as f64,
        (tc_dur.as_micros() as f64 / total_us) * 100.0
    );
    println!(
        "  4. Codegen (JS + .d.ts):       {:>8.2} µs ({:>5.1}%)",
        cg_dur.as_micros() as f64,
        (cg_dur.as_micros() as f64 / total_us) * 100.0
    );
    println!("  ----------------------------------------------------------");
    println!(
        "  TOTAL Latencia Interna:        {:>8.2} µs ({:.3} ms)",
        total_us,
        total_dur.as_secs_f64() * 1000.0
    );

    let loc = source.lines().count() as f64;
    let loc_per_sec = (loc / total_dur.as_secs_f64()) as u64;
    println!(
        "  Throughput Motor Aura:         {:>8} Líneas/segundo",
        loc_per_sec
    );
}

fn bench_batch_scaling() {
    println!("\n============================================================");
    println!("  ESCALABILIDAD EN MEMORIA: PROYECTOS MULTI-MÓDULO (AURA)");
    println!("============================================================");

    let base_module = r#"
module Analytics.Engine

export type EventType =
  | Click { target: String, x: Int, y: Int }
  | PageView { url: String, duration: Float }
  | Purchase { amount: Float, itemsCount: Int }

export type MetricResult = {
  totalEvents: Int,
  totalRevenue: Float,
  avgDuration: Float
}

export fn processEvent(ev: EventType): Float =>
  match ev {
    Click { x, y } => 0.0,
    PageView { duration } => duration,
    Purchase { amount } => amount
  }

export fn calculateTotal(events: List<EventType>, acc: Float = 0.0): Float =>
  match events {
    [] => acc,
    [head, ...tail] => calculateTotal(tail, acc + processEvent(head))
  }
"#;

    let counts = [10, 50, 100, 500, 1000];
    for count in counts {
        let mut modules = Vec::with_capacity(count);
        for i in 0..count {
            modules
                .push(base_module.replace("Analytics.Engine", &format!("Analytics.Engine_{}", i)));
        }

        let total_lines: usize = modules.iter().map(|m| m.lines().count()).sum();

        let iterations = if count <= 100 { 100 } else { 20 };
        let start = Instant::now();
        for _ in 0..iterations {
            for m in &modules {
                let _ = compile(m, &[]).unwrap();
            }
        }
        let dur = start.elapsed() / iterations;
        let ms = dur.as_secs_f64() * 1000.0;
        let loc_per_sec = ((total_lines as f64) / dur.as_secs_f64()) as u64;

        println!(
            "  - {:>4} Módulos ({:>5} LOC): {:>8.2} ms | Throughput: {:>9} LOC/s",
            count, total_lines, ms, loc_per_sec
        );
    }
}

fn main() {
    println!("============================================================");
    println!("   MICRO-BENCHMARKS DE RENDIMIENTO: COMPILADOR AURA (aurac)  ");
    println!("============================================================");

    let eco = fs::read_to_string("examples/ecommerce.aura").expect("ecommerce.aura");
    let backend =
        fs::read_to_string("examples/backend_service.aura").expect("backend_service.aura");
    let api = fs::read_to_string("examples/api_service.aura").expect("api_service.aura");

    let dts = fs::read_to_string("examples/npm_packages.d.ts").unwrap_or_default();

    bench_stages("Ecommerce (77 LOC)", &eco);
    bench_stages("Backend Service (120 LOC)", &backend);

    // Bench with DTS parsing
    println!("\n------------------------------------------------------------");
    println!("  Análisis de Fases Internas: API Service con Ingesta .d.ts");
    println!("------------------------------------------------------------");
    let dts_inputs = [("npm_packages.d.ts", dts.as_str())];
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = compile(&api, &dts_inputs).unwrap();
    }
    let total_dur = start.elapsed() / 1000;
    println!(
        "  TOTAL Latencia (Parsing DTS + Aura + Typecheck + Codegen): {:.3} ms ({:.1} µs)",
        total_dur.as_secs_f64() * 1000.0,
        total_dur.as_micros() as f64
    );
    println!(
        "  Throughput: {:>8} Líneas/segundo",
        ((api.lines().count() as f64) / total_dur.as_secs_f64()) as u64
    );

    bench_batch_scaling();
}
