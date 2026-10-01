//! Type Checker and Bidirectional Type Inference engine for Aura.

use crate::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum VariantFieldTypes {
    Unit,
    Tuple(Vec<ConcreteType>),
    Record(HashMap<String, ConcreteType>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConcreteType {
    Int,
    Float,
    String,
    Bool,
    Unit,
    Option(Box<ConcreteType>),
    Result(Box<ConcreteType>, Box<ConcreteType>),
    Task(Box<ConcreteType>, Box<ConcreteType>),
    Channel(Box<ConcreteType>),
    SendChannel(Box<ConcreteType>),
    RecvChannel(Box<ConcreteType>),
    Pointer(Box<ConcreteType>),
    Interface {
        name: String,
        methods: HashMap<String, (Vec<ConcreteType>, ConcreteType)>,
    },
    List(Box<ConcreteType>),
    Map(Box<ConcreteType>, Box<ConcreteType>),
    Set(Box<ConcreteType>),
    Tuple(Vec<ConcreteType>),
    Record(HashMap<String, ConcreteType>),
    Function {
        params: Vec<ConcreteType>,
        return_type: Box<ConcreteType>,
    },
    SumType {
        name: String,
        type_args: Vec<ConcreteType>,
        variants: HashMap<String, VariantFieldTypes>,
    },
    Generic(Vec<String>, Box<ConcreteType>),
    TypeVar(String),
    Any,
}

impl std::fmt::Display for ConcreteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConcreteType::Int => write!(f, "Int"),
            ConcreteType::Float => write!(f, "Float"),
            ConcreteType::String => write!(f, "String"),
            ConcreteType::Bool => write!(f, "Bool"),
            ConcreteType::Unit => write!(f, "()"),
            ConcreteType::Option(inner) => write!(f, "Option<{}>", inner),
            ConcreteType::Result(ok, err) => write!(f, "Result<{}, {}>", ok, err),
            ConcreteType::Task(ok, err) => write!(f, "Task<{}, {}>", ok, err),
            ConcreteType::Channel(inner) => write!(f, "Channel<{}>", inner),
            ConcreteType::SendChannel(inner) => write!(f, "SendChannel<{}>", inner),
            ConcreteType::RecvChannel(inner) => write!(f, "RecvChannel<{}>", inner),
            ConcreteType::Pointer(inner) => write!(f, "*{}", inner),
            ConcreteType::Interface { name, .. } => write!(f, "interface {}", name),

            ConcreteType::List(elem) => write!(f, "List<{}>", elem),
            ConcreteType::Map(k, v) => write!(f, "Map<{}, {}>", k, v),
            ConcreteType::Set(inner) => write!(f, "Set<{}>", inner),
            ConcreteType::Tuple(types) => {
                write!(f, "(")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
            ConcreteType::Record(fields) => {
                write!(f, "{{ ")?;
                for (k, v) in fields {
                    write!(f, "{}: {}, ", k, v)?;
                }
                write!(f, "}}")
            }
            ConcreteType::Function {
                params,
                return_type,
            } => {
                write!(f, "(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ") -> {}", return_type)
            }
            ConcreteType::SumType {
                name, type_args, ..
            } => {
                if type_args.is_empty() {
                    write!(f, "{}", name)
                } else {
                    write!(f, "{}<", name)?;
                    for (i, p) in type_args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", p)?;
                    }
                    write!(f, ">")
                }
            }
            ConcreteType::Generic(params, inner) => {
                write!(f, "<{}> {}", params.join(", "), inner)
            }
            ConcreteType::TypeVar(v) => write!(f, "{}", v),
            ConcreteType::Any => write!(f, "Any"),
        }
    }
}

impl ConcreteType {
    pub fn substitute(&self, bindings: &HashMap<String, ConcreteType>) -> ConcreteType {
        if bindings.is_empty() {
            return self.clone();
        }
        match self {
            ConcreteType::TypeVar(name) => {
                if let Some(ty) = bindings.get(name) {
                    ty.clone()
                } else {
                    self.clone()
                }
            }
            ConcreteType::Option(inner) => {
                ConcreteType::Option(Box::new(inner.substitute(bindings)))
            }
            ConcreteType::Result(ok, err) => ConcreteType::Result(
                Box::new(ok.substitute(bindings)),
                Box::new(err.substitute(bindings)),
            ),
            ConcreteType::Task(ok, err) => ConcreteType::Task(
                Box::new(ok.substitute(bindings)),
                Box::new(err.substitute(bindings)),
            ),
            ConcreteType::Channel(inner) => {
                ConcreteType::Channel(Box::new(inner.substitute(bindings)))
            }
            ConcreteType::SendChannel(inner) => {
                ConcreteType::SendChannel(Box::new(inner.substitute(bindings)))
            }
            ConcreteType::RecvChannel(inner) => {
                ConcreteType::RecvChannel(Box::new(inner.substitute(bindings)))
            }
            ConcreteType::Pointer(inner) => {
                ConcreteType::Pointer(Box::new(inner.substitute(bindings)))
            }
            ConcreteType::Interface { name, methods } => {
                let mut new_methods = HashMap::new();
                for (mname, (mparams, mret)) in methods {
                    let new_params = mparams.iter().map(|p| p.substitute(bindings)).collect();
                    let new_ret = mret.substitute(bindings);
                    new_methods.insert(mname.clone(), (new_params, new_ret));
                }
                ConcreteType::Interface {
                    name: name.clone(),
                    methods: new_methods,
                }
            }
            ConcreteType::List(inner) => ConcreteType::List(Box::new(inner.substitute(bindings))),
            ConcreteType::Map(k, v) => ConcreteType::Map(
                Box::new(k.substitute(bindings)),
                Box::new(v.substitute(bindings)),
            ),
            ConcreteType::Set(inner) => ConcreteType::Set(Box::new(inner.substitute(bindings))),
            ConcreteType::Tuple(types) => {
                ConcreteType::Tuple(types.iter().map(|t| t.substitute(bindings)).collect())
            }
            ConcreteType::Record(fields) => {
                let mut new_fields = HashMap::new();
                for (k, v) in fields {
                    new_fields.insert(k.clone(), v.substitute(bindings));
                }
                ConcreteType::Record(new_fields)
            }
            ConcreteType::Function {
                params,
                return_type,
            } => ConcreteType::Function {
                params: params.iter().map(|p| p.substitute(bindings)).collect(),
                return_type: Box::new(return_type.substitute(bindings)),
            },
            ConcreteType::SumType {
                name,
                type_args,
                variants,
            } => {
                let new_type_args = type_args.iter().map(|t| t.substitute(bindings)).collect();
                let mut new_variants = HashMap::new();
                for (vname, shape) in variants {
                    let new_shape = match shape {
                        VariantFieldTypes::Unit => VariantFieldTypes::Unit,
                        VariantFieldTypes::Tuple(tys) => VariantFieldTypes::Tuple(
                            tys.iter().map(|t| t.substitute(bindings)).collect(),
                        ),
                        VariantFieldTypes::Record(fields) => {
                            let mut map = HashMap::new();
                            for (fname, ftype) in fields {
                                map.insert(fname.clone(), ftype.substitute(bindings));
                            }
                            VariantFieldTypes::Record(map)
                        }
                    };
                    new_variants.insert(vname.clone(), new_shape);
                }
                ConcreteType::SumType {
                    name: name.clone(),
                    type_args: new_type_args,
                    variants: new_variants,
                }
            }
            ConcreteType::Generic(params, inner) => {
                let mut new_bindings = bindings.clone();
                for p in params {
                    new_bindings.remove(p);
                }
                if new_bindings.is_empty() {
                    self.clone()
                } else {
                    ConcreteType::Generic(params.clone(), Box::new(inner.substitute(&new_bindings)))
                }
            }
            _ => self.clone(),
        }
    }
}

#[derive(Debug)]
pub struct TypeEnv {
    variables: HashMap<String, ConcreteType>,
    types: HashMap<String, ConcreteType>,
    parent: Option<Box<TypeEnv>>,
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeEnv {
    pub fn new() -> Self {
        let mut env = Self {
            variables: HashMap::new(),
            types: HashMap::new(),
            parent: None,
        };

        env.types.insert("Int".to_string(), ConcreteType::Int);
        env.types.insert("Int8".to_string(), ConcreteType::Int);
        env.types.insert("Int16".to_string(), ConcreteType::Int);
        env.types.insert("Int32".to_string(), ConcreteType::Int);
        env.types.insert("Int64".to_string(), ConcreteType::Int);
        env.types.insert("Uint8".to_string(), ConcreteType::Int);
        env.types.insert("Uint16".to_string(), ConcreteType::Int);
        env.types.insert("Uint32".to_string(), ConcreteType::Int);
        env.types.insert("Uint64".to_string(), ConcreteType::Int);
        env.types.insert("Byte".to_string(), ConcreteType::Int);
        env.types.insert("Rune".to_string(), ConcreteType::Int);
        env.types.insert("Uintptr".to_string(), ConcreteType::Int);
        env.types.insert("Float".to_string(), ConcreteType::Float);
        env.types.insert("Float32".to_string(), ConcreteType::Float);
        env.types.insert("Float64".to_string(), ConcreteType::Float);
        env.types.insert("String".to_string(), ConcreteType::String);
        env.types.insert("Bool".to_string(), ConcreteType::Bool);
        env.types.insert("Unit".to_string(), ConcreteType::Unit);

        let list_generic = ConcreteType::Generic(
            vec!["T".to_string()],
            Box::new(ConcreteType::List(Box::new(ConcreteType::TypeVar(
                "T".to_string(),
            )))),
        );
        env.types.insert("List".to_string(), list_generic.clone());
        env.types.insert("Array".to_string(), list_generic);

        let mut iter_methods = HashMap::new();
        iter_methods.insert(
            "next".to_string(),
            (
                vec![],
                ConcreteType::Option(Box::new(ConcreteType::TypeVar("T".to_string()))),
            ),
        );
        iter_methods.insert(
            "map".to_string(),
            (
                vec![ConcreteType::Function {
                    params: vec![ConcreteType::TypeVar("T".to_string())],
                    return_type: Box::new(ConcreteType::TypeVar("U".to_string())),
                }],
                ConcreteType::Interface {
                    name: "Iterator".to_string(),
                    methods: HashMap::new(),
                },
            ),
        );
        iter_methods.insert(
            "filter".to_string(),
            (
                vec![ConcreteType::Function {
                    params: vec![ConcreteType::TypeVar("T".to_string())],
                    return_type: Box::new(ConcreteType::Bool),
                }],
                ConcreteType::Interface {
                    name: "Iterator".to_string(),
                    methods: HashMap::new(),
                },
            ),
        );
        iter_methods.insert(
            "sort".to_string(),
            (
                vec![],
                ConcreteType::Interface {
                    name: "Iterator".to_string(),
                    methods: HashMap::new(),
                },
            ),
        );
        iter_methods.insert(
            "toList".to_string(),
            (
                vec![],
                ConcreteType::List(Box::new(ConcreteType::TypeVar("T".to_string()))),
            ),
        );
        iter_methods.insert(
            "toArray".to_string(),
            (
                vec![],
                ConcreteType::List(Box::new(ConcreteType::TypeVar("T".to_string()))),
            ),
        );
        let iter_iface = ConcreteType::Interface {
            name: "Iterator".to_string(),
            methods: iter_methods,
        };
        let iter_generic = ConcreteType::Generic(vec!["T".to_string()], Box::new(iter_iface));
        env.types
            .insert("Iterator".to_string(), iter_generic.clone());
        env.types.insert("Interator".to_string(), iter_generic);

        env.variables.insert("null".to_string(), ConcreteType::Any);
        env.variables
            .insert("undefined".to_string(), ConcreteType::Any);

        let mut json_methods = HashMap::new();
        json_methods.insert(
            "stringify".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::String),
            },
        );
        json_methods.insert(
            "parse".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String],
                return_type: Box::new(ConcreteType::Any),
            },
        );
        let json_type = ConcreteType::Record(json_methods);
        env.types.insert("JSON".to_string(), json_type.clone());
        env.variables.insert("JSON".to_string(), json_type);

        // Process and File System Builtins
        let mut process_fields = HashMap::new();
        process_fields.insert(
            "argv".to_string(),
            ConcreteType::List(Box::new(ConcreteType::String)),
        );
        process_fields.insert(
            "exit".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Int],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        process_fields.insert(
            "cwd".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::String),
            },
        );
        process_fields.insert(
            "uptime".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Float),
            },
        );
        process_fields.insert(
            "env".to_string(),
            ConcreteType::Map(
                Box::new(ConcreteType::String),
                Box::new(ConcreteType::String),
            ),
        );
        let process_type = ConcreteType::Record(process_fields);
        env.types
            .insert("Process".to_string(), process_type.clone());
        env.variables.insert("process".to_string(), process_type);

        let mut fs_methods = HashMap::new();
        fs_methods.insert(
            "readFileSync".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::String],
                return_type: Box::new(ConcreteType::String),
            },
        );
        fs_methods.insert(
            "writeFileSync".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::String],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        fs_methods.insert(
            "existsSync".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String],
                return_type: Box::new(ConcreteType::Bool),
            },
        );
        fs_methods.insert(
            "mkdirSync".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        fs_methods.insert(
            "readdirSync".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String],
                return_type: Box::new(ConcreteType::List(Box::new(ConcreteType::String))),
            },
        );
        let fs_type = ConcreteType::Record(fs_methods);
        env.types.insert("Fs".to_string(), fs_type.clone());
        env.variables.insert("fs".to_string(), fs_type);

        // Backend Go-style OS and Systems Primitives
        let mut os_methods = HashMap::new();
        os_methods.insert(
            "args".to_string(),
            ConcreteType::List(Box::new(ConcreteType::String)),
        );
        os_methods.insert(
            "env".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String],
                return_type: Box::new(ConcreteType::String),
            },
        );
        os_methods.insert(
            "getEnv".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String],
                return_type: Box::new(ConcreteType::String),
            },
        );
        os_methods.insert(
            "setEnv".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::String],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        os_methods.insert(
            "exit".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Int],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        os_methods.insert(
            "readFile".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String],
                return_type: Box::new(ConcreteType::String),
            },
        );
        os_methods.insert(
            "writeFile".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::String],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        os_methods.insert(
            "hostname".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::String),
            },
        );
        let os_type = ConcreteType::Record(os_methods);
        env.types.insert("OS".to_string(), os_type.clone());
        env.variables.insert("os".to_string(), os_type);

        let mut time_methods = HashMap::new();
        time_methods.insert(
            "now".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Int),
            },
        );
        time_methods.insert(
            "isoString".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::String),
            },
        );
        time_methods.insert(
            "sleep".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Int],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Unit),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        let time_type = ConcreteType::Record(time_methods);
        env.types.insert("Time".to_string(), time_type.clone());
        env.variables.insert("time".to_string(), time_type);

        // Crypto standard module
        let mut crypto_methods = HashMap::new();
        crypto_methods.insert(
            "hmacSha256".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::String],
                return_type: Box::new(ConcreteType::String),
            },
        );
        crypto_methods.insert(
            "sha256".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String],
                return_type: Box::new(ConcreteType::String),
            },
        );
        crypto_methods.insert(
            "base64UrlEncode".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String],
                return_type: Box::new(ConcreteType::String),
            },
        );
        crypto_methods.insert(
            "base64UrlDecode".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String],
                return_type: Box::new(ConcreteType::Result(
                    Box::new(ConcreteType::String),
                    Box::new(ConcreteType::String),
                )),
            },
        );
        let crypto_type = ConcreteType::Record(crypto_methods);
        env.types.insert("Crypto".to_string(), crypto_type.clone());
        env.variables.insert("crypto".to_string(), crypto_type);

        // JWT standard module
        let mut jwt_methods = HashMap::new();
        jwt_methods.insert(
            "sign".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::String],
                return_type: Box::new(ConcreteType::String),
            },
        );
        jwt_methods.insert(
            "verify".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::String],
                return_type: Box::new(ConcreteType::Result(
                    Box::new(ConcreteType::Any),
                    Box::new(ConcreteType::String),
                )),
            },
        );
        let jwt_type = ConcreteType::Record(jwt_methods);
        env.types.insert("Jwt".to_string(), jwt_type.clone());
        env.variables.insert("jwt".to_string(), jwt_type);

        // Collections constructors
        env.variables.insert(
            "Map".to_string(),
            ConcreteType::Record({
                let mut f = HashMap::new();
                f.insert(
                    "new".to_string(),
                    ConcreteType::Function {
                        params: vec![],
                        return_type: Box::new(ConcreteType::Map(
                            Box::new(ConcreteType::TypeVar("K".to_string())),
                            Box::new(ConcreteType::TypeVar("V".to_string())),
                        )),
                    },
                );
                f
            }),
        );
        env.variables.insert(
            "Set".to_string(),
            ConcreteType::Record({
                let mut f = HashMap::new();
                f.insert(
                    "new".to_string(),
                    ConcreteType::Function {
                        params: vec![],
                        return_type: Box::new(ConcreteType::Set(Box::new(ConcreteType::TypeVar(
                            "T".to_string(),
                        )))),
                    },
                );
                f
            }),
        );

        // Concurrency (CSP) Primitives
        env.variables.insert(
            "Channel".to_string(),
            ConcreteType::Record({
                let mut f = HashMap::new();
                f.insert(
                    "make".to_string(),
                    ConcreteType::Function {
                        params: vec![],
                        return_type: Box::new(ConcreteType::Channel(Box::new(
                            ConcreteType::TypeVar("T".to_string()),
                        ))),
                    },
                );
                f.insert(
                    "new".to_string(),
                    ConcreteType::Function {
                        params: vec![],
                        return_type: Box::new(ConcreteType::Channel(Box::new(
                            ConcreteType::TypeVar("T".to_string()),
                        ))),
                    },
                );
                f.insert(
                    "close".to_string(),
                    ConcreteType::Function {
                        params: vec![ConcreteType::Channel(Box::new(ConcreteType::TypeVar(
                            "T".to_string(),
                        )))],
                        return_type: Box::new(ConcreteType::Unit),
                    },
                );
                f
            }),
        );

        env.variables.insert(
            "WaitGroup".to_string(),
            ConcreteType::Record({
                let mut f = HashMap::new();
                f.insert(
                    "new".to_string(),
                    ConcreteType::Function {
                        params: vec![],
                        return_type: Box::new(ConcreteType::Record({
                            let mut wf = HashMap::new();
                            wf.insert(
                                "add".to_string(),
                                ConcreteType::Function {
                                    params: vec![ConcreteType::Int],
                                    return_type: Box::new(ConcreteType::Unit),
                                },
                            );
                            wf.insert(
                                "done".to_string(),
                                ConcreteType::Function {
                                    params: vec![],
                                    return_type: Box::new(ConcreteType::Unit),
                                },
                            );
                            wf.insert(
                                "wait".to_string(),
                                ConcreteType::Function {
                                    params: vec![],
                                    return_type: Box::new(ConcreteType::Task(
                                        Box::new(ConcreteType::Unit),
                                        Box::new(ConcreteType::Any),
                                    )),
                                },
                            );
                            wf
                        })),
                    },
                );
                f
            }),
        );

        let mutex_type = ConcreteType::Record({
            let mut mf = HashMap::new();
            mf.insert(
                "lock".to_string(),
                ConcreteType::Function {
                    params: vec![],
                    return_type: Box::new(ConcreteType::Task(
                        Box::new(ConcreteType::Unit),
                        Box::new(ConcreteType::Any),
                    )),
                },
            );
            mf.insert(
                "unlock".to_string(),
                ConcreteType::Function {
                    params: vec![],
                    return_type: Box::new(ConcreteType::Unit),
                },
            );
            mf.insert(
                "tryLock".to_string(),
                ConcreteType::Function {
                    params: vec![],
                    return_type: Box::new(ConcreteType::Bool),
                },
            );
            mf
        });
        env.types.insert("Mutex".to_string(), mutex_type.clone());
        let mutex_ctor = ConcreteType::Record({
            let mut f = HashMap::new();
            f.insert(
                "new".to_string(),
                ConcreteType::Function {
                    params: vec![],
                    return_type: Box::new(mutex_type.clone()),
                },
            );
            f
        });
        env.variables
            .insert("Mutex".to_string(), mutex_ctor.clone());

        let rwmutex_type = ConcreteType::Record({
            let mut mf = HashMap::new();
            mf.insert(
                "lock".to_string(),
                ConcreteType::Function {
                    params: vec![],
                    return_type: Box::new(ConcreteType::Task(
                        Box::new(ConcreteType::Unit),
                        Box::new(ConcreteType::Any),
                    )),
                },
            );
            mf.insert(
                "unlock".to_string(),
                ConcreteType::Function {
                    params: vec![],
                    return_type: Box::new(ConcreteType::Unit),
                },
            );
            mf.insert(
                "rLock".to_string(),
                ConcreteType::Function {
                    params: vec![],
                    return_type: Box::new(ConcreteType::Task(
                        Box::new(ConcreteType::Unit),
                        Box::new(ConcreteType::Any),
                    )),
                },
            );
            mf.insert(
                "rUnlock".to_string(),
                ConcreteType::Function {
                    params: vec![],
                    return_type: Box::new(ConcreteType::Unit),
                },
            );
            mf
        });
        env.types
            .insert("RWMutex".to_string(), rwmutex_type.clone());
        let rwmutex_ctor = ConcreteType::Record({
            let mut f = HashMap::new();
            f.insert(
                "new".to_string(),
                ConcreteType::Function {
                    params: vec![],
                    return_type: Box::new(rwmutex_type.clone()),
                },
            );
            f
        });
        env.variables
            .insert("RWMutex".to_string(), rwmutex_ctor.clone());

        let once_type = ConcreteType::Record({
            let mut mf = HashMap::new();
            mf.insert(
                "do".to_string(),
                ConcreteType::Function {
                    params: vec![ConcreteType::Function {
                        params: vec![],
                        return_type: Box::new(ConcreteType::Unit),
                    }],
                    return_type: Box::new(ConcreteType::Unit),
                },
            );
            mf
        });
        env.types.insert("Once".to_string(), once_type.clone());
        let once_ctor = ConcreteType::Record({
            let mut f = HashMap::new();
            f.insert(
                "new".to_string(),
                ConcreteType::Function {
                    params: vec![],
                    return_type: Box::new(once_type.clone()),
                },
            );
            f
        });
        env.variables.insert("Once".to_string(), once_ctor.clone());

        let pool_type = ConcreteType::Record({
            let mut pf = HashMap::new();
            pf.insert(
                "get".to_string(),
                ConcreteType::Function {
                    params: vec![],
                    return_type: Box::new(ConcreteType::Any),
                },
            );
            pf.insert(
                "put".to_string(),
                ConcreteType::Function {
                    params: vec![ConcreteType::Any],
                    return_type: Box::new(ConcreteType::Unit),
                },
            );
            pf
        });
        env.types.insert("Pool".to_string(), pool_type.clone());
        let pool_ctor = ConcreteType::Record({
            let mut f = HashMap::new();
            f.insert(
                "new".to_string(),
                ConcreteType::Function {
                    params: vec![ConcreteType::Function {
                        params: vec![],
                        return_type: Box::new(ConcreteType::Any),
                    }],
                    return_type: Box::new(pool_type.clone()),
                },
            );
            f
        });
        env.variables.insert("Pool".to_string(), pool_ctor.clone());

        let wg_ctor = env.variables.get("WaitGroup").unwrap().clone();
        env.variables.insert(
            "Sync".to_string(),
            ConcreteType::Record({
                let mut sf = HashMap::new();
                sf.insert("Mutex".to_string(), mutex_ctor);
                sf.insert("RWMutex".to_string(), rwmutex_ctor);
                sf.insert("Once".to_string(), once_ctor);
                sf.insert("Pool".to_string(), pool_ctor);
                sf.insert("WaitGroup".to_string(), wg_ctor);
                sf
            }),
        );

        env.variables.insert(
            "close".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        env.variables.insert(
            "panic".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        env.variables.insert(
            "recover".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Option(Box::new(ConcreteType::Any))),
            },
        );
        env.variables.insert(
            "ptr".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::TypeVar("T".to_string())],
                return_type: Box::new(ConcreteType::Pointer(Box::new(ConcreteType::TypeVar(
                    "T".to_string(),
                )))),
            },
        );
        env.variables.insert(
            "StructTag".to_string(),
            ConcreteType::Record({
                let mut stf = HashMap::new();
                stf.insert(
                    "get".to_string(),
                    ConcreteType::Function {
                        params: vec![
                            ConcreteType::Any,
                            ConcreteType::String,
                            ConcreteType::String,
                        ],
                        return_type: Box::new(ConcreteType::Option(Box::new(ConcreteType::String))),
                    },
                );
                stf
            }),
        );

        env.variables.insert(
            "timeout".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Int],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Unit),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        env.variables.insert(
            "sleep".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Int],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Unit),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        env.variables.insert(
            "print".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        env.variables.insert(
            "println".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        env.variables.insert(
            "yield".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        env.variables.insert(
            "parseInt".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String],
                return_type: Box::new(ConcreteType::Int),
            },
        );
        env.variables.insert(
            "parseFloat".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String],
                return_type: Box::new(ConcreteType::Float),
            },
        );
        env.variables.insert(
            "len".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::Int),
            },
        );
        env.variables.insert(
            "cap".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::Int),
            },
        );
        env.variables.insert(
            "append".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Any),
            },
        );
        env.variables.insert(
            "make".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::Any),
            },
        );

        // Testing & Assertion Primitives (Go-test style)
        let mut t_methods = HashMap::new();
        t_methods.insert(
            "assert".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Bool, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        t_methods.insert(
            "assertEqual".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::Any, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        t_methods.insert(
            "assertNotEqual".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::Any, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        t_methods.insert(
            "assertDeepEqual".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::Any, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        t_methods.insert(
            "assertTrue".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Bool, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        t_methods.insert(
            "assertFalse".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Bool, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        t_methods.insert(
            "assertThrows".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        t_methods.insert(
            "fail".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        t_methods.insert(
            "log".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        t_methods.insert(
            "skip".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        t_methods.insert(
            "step".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Any),
            },
        );
        t_methods.insert(
            "run".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Any),
            },
        );
        let testing_t_type = ConcreteType::Record(t_methods);
        env.types
            .insert("TestingT".to_string(), testing_t_type.clone());
        env.types
            .insert("TestingContext".to_string(), testing_t_type.clone());
        env.types.insert("T".to_string(), testing_t_type);

        let mut b_methods = HashMap::new();
        b_methods.insert("n".to_string(), ConcreteType::Int);
        b_methods.insert(
            "resetTimer".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        b_methods.insert(
            "startTimer".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        b_methods.insert(
            "stopTimer".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        b_methods.insert(
            "setBytes".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Int],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        b_methods.insert(
            "reportAllocs".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        let benchmark_b_type = ConcreteType::Record(b_methods);
        env.types
            .insert("BenchmarkB".to_string(), benchmark_b_type.clone());
        env.types
            .insert("BenchmarkContext".to_string(), benchmark_b_type.clone());
        env.types.insert("B".to_string(), benchmark_b_type);

        // Global assertion helpers
        env.variables.insert(
            "assert".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Bool, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        env.variables.insert(
            "assertEqual".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::Any, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        env.variables.insert(
            "assertNotEqual".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::Any, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        env.variables.insert(
            "assertDeepEqual".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::Any, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        env.variables.insert(
            "assertTrue".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Bool, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        env.variables.insert(
            "assertFalse".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Bool, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        env.variables.insert(
            "assertThrows".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );

        // Global fetch API
        env.variables.insert(
            "fetch".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Any),
                    Box::new(ConcreteType::Any),
                )),
            },
        );

        // Global collection and iterator transformation functions
        env.variables.insert(
            "map".to_string(),
            ConcreteType::Function {
                params: vec![
                    ConcreteType::TypeVar("Collection".to_string()),
                    ConcreteType::Function {
                        params: vec![ConcreteType::TypeVar("T".to_string())],
                        return_type: Box::new(ConcreteType::TypeVar("U".to_string())),
                    },
                ],
                return_type: Box::new(ConcreteType::TypeVar("ResultCollection".to_string())),
            },
        );
        env.variables.insert(
            "filter".to_string(),
            ConcreteType::Function {
                params: vec![
                    ConcreteType::TypeVar("Collection".to_string()),
                    ConcreteType::Function {
                        params: vec![ConcreteType::TypeVar("T".to_string())],
                        return_type: Box::new(ConcreteType::Bool),
                    },
                ],
                return_type: Box::new(ConcreteType::TypeVar("Collection".to_string())),
            },
        );
        env.variables.insert(
            "sort".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::TypeVar("Collection".to_string())],
                return_type: Box::new(ConcreteType::TypeVar("Collection".to_string())),
            },
        );
        env.variables.insert(
            "iter".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::TypeVar("Collection".to_string())],
                return_type: Box::new(ConcreteType::TypeVar("Iterator".to_string())),
            },
        );
        env.variables.insert(
            "toList".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::TypeVar("Collection".to_string())],
                return_type: Box::new(ConcreteType::List(Box::new(ConcreteType::TypeVar(
                    "T".to_string(),
                )))),
            },
        );
        env.variables.insert(
            "toArray".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::TypeVar("Collection".to_string())],
                return_type: Box::new(ConcreteType::List(Box::new(ConcreteType::TypeVar(
                    "T".to_string(),
                )))),
            },
        );

        // HTTP Standard Server (Golang style - net/http and ServeMux)
        let mut serve_mux_methods = HashMap::new();
        serve_mux_methods.insert(
            "handleFunc".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        serve_mux_methods.insert(
            "handle".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        serve_mux_methods.insert(
            "get".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        serve_mux_methods.insert(
            "post".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        serve_mux_methods.insert(
            "put".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        serve_mux_methods.insert(
            "delete".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        serve_mux_methods.insert(
            "patch".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        serve_mux_methods.insert(
            "doc".to_string(),
            ConcreteType::Function {
                params: vec![
                    ConcreteType::String,
                    ConcreteType::String,
                    ConcreteType::Any,
                ],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        serve_mux_methods.insert(
            "Doc".to_string(),
            ConcreteType::Function {
                params: vec![
                    ConcreteType::String,
                    ConcreteType::String,
                    ConcreteType::Any,
                ],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        serve_mux_methods.insert(
            "document".to_string(),
            ConcreteType::Function {
                params: vec![
                    ConcreteType::String,
                    ConcreteType::String,
                    ConcreteType::Any,
                ],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        serve_mux_methods.insert(
            "Document".to_string(),
            ConcreteType::Function {
                params: vec![
                    ConcreteType::String,
                    ConcreteType::String,
                    ConcreteType::Any,
                ],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        serve_mux_methods.insert(
            "use".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        serve_mux_methods.insert(
            "enableSwagger".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::Any, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        serve_mux_methods.insert(
            "EnableSwagger".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::Any, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        serve_mux_methods.insert(
            "swagger".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::Any, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        serve_mux_methods.insert(
            "Swagger".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::Any, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        serve_mux_methods.insert(
            "swaggerUI".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::Any, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        serve_mux_methods.insert(
            "openAPI".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::String),
            },
        );
        serve_mux_methods.insert(
            "OpenAPI".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::String),
            },
        );
        serve_mux_methods.insert(
            "listenAndServe".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Unit),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        serve_mux_methods.insert(
            "serveHTTP".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Unit),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        serve_mux_methods.insert(
            "close".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Unit),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        let serve_mux_type = ConcreteType::Record(serve_mux_methods.clone());
        env.types
            .insert("ServeMux".to_string(), serve_mux_type.clone());

        let mut http_methods = serve_mux_methods.clone();
        http_methods.insert(
            "newServeMux".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(serve_mux_type.clone()),
            },
        );
        http_methods.insert(
            "NewServeMux".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(serve_mux_type.clone()),
            },
        );
        http_methods.insert(
            "json".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::Int, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        http_methods.insert(
            "text".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::Int, ConcreteType::String],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        http_methods.insert(
            "html".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::Int, ConcreteType::String],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        http_methods.insert(
            "error".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::String, ConcreteType::Int],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        http_methods.insert(
            "redirect".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::String, ConcreteType::Int],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        http_methods.insert(
            "parseJson".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Any),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::String),
                )),
            },
        );
        http_methods.insert(
            "pathValue".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::String],
                return_type: Box::new(ConcreteType::String),
            },
        );
        http_methods.insert(
            "query".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::String],
                return_type: Box::new(ConcreteType::String),
            },
        );
        http_methods.insert(
            "header".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::String],
                return_type: Box::new(ConcreteType::String),
            },
        );
        http_methods.insert(
            "getHeader".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::String],
                return_type: Box::new(ConcreteType::String),
            },
        );
        http_methods.insert(
            "serveFile".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::Any, ConcreteType::String],
                return_type: Box::new(ConcreteType::Unit),
            },
        );
        http_methods.insert(
            "fileServer".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String],
                return_type: Box::new(ConcreteType::Function {
                    params: vec![ConcreteType::Any, ConcreteType::Any],
                    return_type: Box::new(ConcreteType::Unit),
                }),
            },
        );

        // HTTP Status Constants
        http_methods.insert("StatusOK".to_string(), ConcreteType::Int);
        http_methods.insert("StatusCreated".to_string(), ConcreteType::Int);
        http_methods.insert("StatusAccepted".to_string(), ConcreteType::Int);
        http_methods.insert("StatusNoContent".to_string(), ConcreteType::Int);
        http_methods.insert("StatusMovedPermanently".to_string(), ConcreteType::Int);
        http_methods.insert("StatusFound".to_string(), ConcreteType::Int);
        http_methods.insert("StatusBadRequest".to_string(), ConcreteType::Int);
        http_methods.insert("StatusUnauthorized".to_string(), ConcreteType::Int);
        http_methods.insert("StatusForbidden".to_string(), ConcreteType::Int);
        http_methods.insert("StatusNotFound".to_string(), ConcreteType::Int);
        http_methods.insert("StatusMethodNotAllowed".to_string(), ConcreteType::Int);
        http_methods.insert("StatusConflict".to_string(), ConcreteType::Int);
        http_methods.insert("StatusUnprocessableEntity".to_string(), ConcreteType::Int);
        http_methods.insert("StatusInternalServerError".to_string(), ConcreteType::Int);
        http_methods.insert("StatusBadGateway".to_string(), ConcreteType::Int);
        http_methods.insert("StatusServiceUnavailable".to_string(), ConcreteType::Int);

        let http_type = ConcreteType::Record(http_methods);
        env.types
            .insert("HttpServer".to_string(), http_type.clone());
        env.variables.insert("http".to_string(), http_type);

        // MySQL Database Standard Library Integration
        let mut mysql_query_result_fields = HashMap::new();
        mysql_query_result_fields.insert("affectedRows".to_string(), ConcreteType::Int);
        mysql_query_result_fields.insert("insertId".to_string(), ConcreteType::Int);
        mysql_query_result_fields.insert("changedRows".to_string(), ConcreteType::Int);
        mysql_query_result_fields.insert("warningCount".to_string(), ConcreteType::Int);
        let mysql_query_result_type = ConcreteType::Record(mysql_query_result_fields);
        env.types.insert(
            "MysqlQueryResult".to_string(),
            mysql_query_result_type.clone(),
        );

        let mut mysql_config_fields = HashMap::new();
        mysql_config_fields.insert("host".to_string(), ConcreteType::String);
        mysql_config_fields.insert("port".to_string(), ConcreteType::Int);
        mysql_config_fields.insert("user".to_string(), ConcreteType::String);
        mysql_config_fields.insert("password".to_string(), ConcreteType::String);
        mysql_config_fields.insert("database".to_string(), ConcreteType::String);
        mysql_config_fields.insert("uri".to_string(), ConcreteType::String);
        mysql_config_fields.insert("connectionLimit".to_string(), ConcreteType::Int);
        let mysql_config_type = ConcreteType::Record(mysql_config_fields);
        env.types
            .insert("MysqlConfig".to_string(), mysql_config_type);

        let mut tx_methods = HashMap::new();
        tx_methods.insert(
            "query".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::List(Box::new(ConcreteType::Any))),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        tx_methods.insert(
            "execute".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(mysql_query_result_type.clone()),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        tx_methods.insert(
            "queryRow".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Option(Box::new(ConcreteType::Any))),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        tx_methods.insert(
            "commit".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Unit),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        tx_methods.insert(
            "rollback".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Unit),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        let mysql_tx_type = ConcreteType::Record(tx_methods);
        env.types
            .insert("MysqlTransaction".to_string(), mysql_tx_type.clone());

        let mut pool_methods = HashMap::new();
        pool_methods.insert(
            "query".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::List(Box::new(ConcreteType::Any))),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        pool_methods.insert(
            "execute".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(mysql_query_result_type),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        pool_methods.insert(
            "queryRow".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Option(Box::new(ConcreteType::Any))),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        pool_methods.insert(
            "transaction".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Function {
                    params: vec![mysql_tx_type],
                    return_type: Box::new(ConcreteType::Any),
                }],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Any),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        pool_methods.insert(
            "ping".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Bool),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        pool_methods.insert(
            "close".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Unit),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        pool_methods.insert(
            "end".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Unit),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        pool_methods.insert(
            "escape".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::String),
            },
        );
        pool_methods.insert(
            "format".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any],
                return_type: Box::new(ConcreteType::String),
            },
        );
        let mysql_pool_type = ConcreteType::Record(pool_methods);
        env.types
            .insert("MysqlPool".to_string(), mysql_pool_type.clone());
        env.types
            .insert("MysqlConnection".to_string(), mysql_pool_type.clone());

        let mut mysql_module_methods = HashMap::new();
        mysql_module_methods.insert(
            "createPool".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(mysql_pool_type.clone()),
            },
        );
        mysql_module_methods.insert(
            "createConnection".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(mysql_pool_type.clone()),
            },
        );
        mysql_module_methods.insert(
            "open".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(mysql_pool_type),
            },
        );
        mysql_module_methods.insert(
            "escape".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::String),
            },
        );
        mysql_module_methods.insert(
            "format".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any],
                return_type: Box::new(ConcreteType::String),
            },
        );
        let mysql_module_type = ConcreteType::Record(mysql_module_methods);
        env.types
            .insert("MysqlModule".to_string(), mysql_module_type.clone());
        env.variables.insert("mysql".to_string(), mysql_module_type);

        // PostgreSQL Database Standard Library Integration
        let mut pg_query_result_fields = HashMap::new();
        pg_query_result_fields.insert("rowCount".to_string(), ConcreteType::Int);
        pg_query_result_fields.insert("command".to_string(), ConcreteType::String);
        pg_query_result_fields.insert("insertId".to_string(), ConcreteType::Int);
        pg_query_result_fields.insert("affectedRows".to_string(), ConcreteType::Int);
        let pg_query_result_type = ConcreteType::Record(pg_query_result_fields);
        env.types
            .insert("PgQueryResult".to_string(), pg_query_result_type.clone());
        env.types.insert(
            "PostgresQueryResult".to_string(),
            pg_query_result_type.clone(),
        );

        let mut pg_config_fields = HashMap::new();
        pg_config_fields.insert("host".to_string(), ConcreteType::String);
        pg_config_fields.insert("port".to_string(), ConcreteType::Int);
        pg_config_fields.insert("user".to_string(), ConcreteType::String);
        pg_config_fields.insert("password".to_string(), ConcreteType::String);
        pg_config_fields.insert("database".to_string(), ConcreteType::String);
        pg_config_fields.insert("uri".to_string(), ConcreteType::String);
        pg_config_fields.insert("connectionLimit".to_string(), ConcreteType::Int);
        let pg_config_type = ConcreteType::Record(pg_config_fields);
        env.types
            .insert("PgConfig".to_string(), pg_config_type.clone());
        env.types
            .insert("PostgresConfig".to_string(), pg_config_type);

        let mut pg_tx_methods = HashMap::new();
        pg_tx_methods.insert(
            "query".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::List(Box::new(ConcreteType::Any))),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        pg_tx_methods.insert(
            "execute".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(pg_query_result_type.clone()),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        pg_tx_methods.insert(
            "queryRow".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Option(Box::new(ConcreteType::Any))),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        pg_tx_methods.insert(
            "commit".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Unit),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        pg_tx_methods.insert(
            "rollback".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Unit),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        let pg_tx_type = ConcreteType::Record(pg_tx_methods);
        env.types
            .insert("PgTransaction".to_string(), pg_tx_type.clone());
        env.types
            .insert("PostgresTransaction".to_string(), pg_tx_type.clone());

        let mut pg_pool_methods = HashMap::new();
        pg_pool_methods.insert(
            "query".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::List(Box::new(ConcreteType::Any))),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        pg_pool_methods.insert(
            "execute".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(pg_query_result_type),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        pg_pool_methods.insert(
            "queryRow".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Option(Box::new(ConcreteType::Any))),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        pg_pool_methods.insert(
            "transaction".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Function {
                    params: vec![pg_tx_type],
                    return_type: Box::new(ConcreteType::Any),
                }],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Any),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        pg_pool_methods.insert(
            "ping".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Bool),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        pg_pool_methods.insert(
            "close".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Unit),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        pg_pool_methods.insert(
            "end".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Unit),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        pg_pool_methods.insert(
            "escapeIdentifier".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::String),
            },
        );
        pg_pool_methods.insert(
            "escapeLiteral".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::String),
            },
        );
        pg_pool_methods.insert(
            "escape".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::String),
            },
        );
        pg_pool_methods.insert(
            "format".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any],
                return_type: Box::new(ConcreteType::String),
            },
        );
        let pg_pool_type = ConcreteType::Record(pg_pool_methods);
        env.types.insert("PgPool".to_string(), pg_pool_type.clone());
        env.types
            .insert("PostgresPool".to_string(), pg_pool_type.clone());
        env.types
            .insert("PgClient".to_string(), pg_pool_type.clone());
        env.types
            .insert("PostgresClient".to_string(), pg_pool_type.clone());

        let mut pg_module_methods = HashMap::new();
        pg_module_methods.insert(
            "createPool".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(pg_pool_type.clone()),
            },
        );
        pg_module_methods.insert(
            "createClient".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(pg_pool_type.clone()),
            },
        );
        pg_module_methods.insert(
            "open".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(pg_pool_type),
            },
        );
        pg_module_methods.insert(
            "escapeIdentifier".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::String),
            },
        );
        pg_module_methods.insert(
            "escapeLiteral".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::String),
            },
        );
        pg_module_methods.insert(
            "escape".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::String),
            },
        );
        pg_module_methods.insert(
            "format".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any],
                return_type: Box::new(ConcreteType::String),
            },
        );
        let pg_module_type = ConcreteType::Record(pg_module_methods);
        env.types
            .insert("PgModule".to_string(), pg_module_type.clone());
        env.types
            .insert("PostgresModule".to_string(), pg_module_type.clone());
        env.variables
            .insert("postgres".to_string(), pg_module_type.clone());
        env.variables.insert("pg".to_string(), pg_module_type);

        // MongoDB Database Standard Library Integration
        let mut mongo_insert_result_fields = HashMap::new();
        mongo_insert_result_fields.insert("insertedId".to_string(), ConcreteType::Any);
        mongo_insert_result_fields.insert("acknowledged".to_string(), ConcreteType::Bool);
        let mongo_insert_result_type = ConcreteType::Record(mongo_insert_result_fields);
        env.types.insert(
            "MongoInsertResult".to_string(),
            mongo_insert_result_type.clone(),
        );

        let mut mongo_insert_many_result_fields = HashMap::new();
        mongo_insert_many_result_fields.insert(
            "insertedIds".to_string(),
            ConcreteType::List(Box::new(ConcreteType::Any)),
        );
        mongo_insert_many_result_fields.insert("insertedCount".to_string(), ConcreteType::Int);
        mongo_insert_many_result_fields.insert("acknowledged".to_string(), ConcreteType::Bool);
        let mongo_insert_many_result_type = ConcreteType::Record(mongo_insert_many_result_fields);
        env.types.insert(
            "MongoInsertManyResult".to_string(),
            mongo_insert_many_result_type.clone(),
        );

        let mut mongo_update_result_fields = HashMap::new();
        mongo_update_result_fields.insert("matchedCount".to_string(), ConcreteType::Int);
        mongo_update_result_fields.insert("modifiedCount".to_string(), ConcreteType::Int);
        mongo_update_result_fields.insert("upsertedId".to_string(), ConcreteType::Any);
        mongo_update_result_fields.insert("acknowledged".to_string(), ConcreteType::Bool);
        let mongo_update_result_type = ConcreteType::Record(mongo_update_result_fields);
        env.types.insert(
            "MongoUpdateResult".to_string(),
            mongo_update_result_type.clone(),
        );

        let mut mongo_delete_result_fields = HashMap::new();
        mongo_delete_result_fields.insert("deletedCount".to_string(), ConcreteType::Int);
        mongo_delete_result_fields.insert("acknowledged".to_string(), ConcreteType::Bool);
        let mongo_delete_result_type = ConcreteType::Record(mongo_delete_result_fields);
        env.types.insert(
            "MongoDeleteResult".to_string(),
            mongo_delete_result_type.clone(),
        );

        let mut mongo_config_fields = HashMap::new();
        mongo_config_fields.insert("uri".to_string(), ConcreteType::String);
        mongo_config_fields.insert("host".to_string(), ConcreteType::String);
        mongo_config_fields.insert("port".to_string(), ConcreteType::Int);
        mongo_config_fields.insert("user".to_string(), ConcreteType::String);
        mongo_config_fields.insert("password".to_string(), ConcreteType::String);
        mongo_config_fields.insert("database".to_string(), ConcreteType::String);
        let mongo_config_type = ConcreteType::Record(mongo_config_fields);
        env.types
            .insert("MongoConfig".to_string(), mongo_config_type);

        let mut mongo_collection_methods = HashMap::new();
        mongo_collection_methods.insert(
            "find".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::List(Box::new(ConcreteType::Any))),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        mongo_collection_methods.insert(
            "findOne".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Option(Box::new(ConcreteType::Any))),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        mongo_collection_methods.insert(
            "insertOne".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(mongo_insert_result_type),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        mongo_collection_methods.insert(
            "insertMany".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(mongo_insert_many_result_type),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        mongo_collection_methods.insert(
            "updateOne".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(mongo_update_result_type.clone()),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        mongo_collection_methods.insert(
            "updateMany".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(mongo_update_result_type),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        mongo_collection_methods.insert(
            "deleteOne".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(mongo_delete_result_type.clone()),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        mongo_collection_methods.insert(
            "deleteMany".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(mongo_delete_result_type),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        mongo_collection_methods.insert(
            "countDocuments".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Int),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        mongo_collection_methods.insert(
            "aggregate".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::List(Box::new(ConcreteType::Any))),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        mongo_collection_methods.insert(
            "drop".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Bool),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        let mongo_collection_type = ConcreteType::Record(mongo_collection_methods);
        env.types
            .insert("MongoCollection".to_string(), mongo_collection_type.clone());

        let mut mongo_db_methods = HashMap::new();
        mongo_db_methods.insert(
            "collection".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String],
                return_type: Box::new(mongo_collection_type.clone()),
            },
        );
        mongo_db_methods.insert(
            "listCollections".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::List(Box::new(ConcreteType::Any))),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        mongo_db_methods.insert(
            "dropDatabase".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Bool),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        let mongo_db_type = ConcreteType::Record(mongo_db_methods);
        env.types
            .insert("MongoDatabase".to_string(), mongo_db_type.clone());

        let mut mongo_client_methods = HashMap::new();
        mongo_client_methods.insert(
            "db".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(mongo_db_type.clone()),
            },
        );
        mongo_client_methods.insert(
            "collection".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String],
                return_type: Box::new(mongo_collection_type),
            },
        );
        mongo_client_methods.insert(
            "ping".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Bool),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        mongo_client_methods.insert(
            "close".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Unit),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        let mongo_client_type = ConcreteType::Record(mongo_client_methods);
        env.types
            .insert("MongoClient".to_string(), mongo_client_type.clone());

        let mut mongo_module_methods = HashMap::new();
        mongo_module_methods.insert(
            "connect".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(mongo_client_type.clone()),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        mongo_module_methods.insert(
            "createClient".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(mongo_client_type.clone()),
            },
        );
        mongo_module_methods.insert(
            "open".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(mongo_client_type),
            },
        );
        mongo_module_methods.insert(
            "db".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(mongo_db_type),
            },
        );
        let mongo_module_type = ConcreteType::Record(mongo_module_methods);
        env.types
            .insert("MongoModule".to_string(), mongo_module_type.clone());
        env.variables
            .insert("mongodb".to_string(), mongo_module_type.clone());
        env.variables.insert("mongo".to_string(), mongo_module_type);

        // Redis Database Standard Library Integration
        let mut redis_config_fields = HashMap::new();
        redis_config_fields.insert("host".to_string(), ConcreteType::String);
        redis_config_fields.insert("port".to_string(), ConcreteType::Int);
        redis_config_fields.insert("password".to_string(), ConcreteType::String);
        redis_config_fields.insert("db".to_string(), ConcreteType::Int);
        redis_config_fields.insert("uri".to_string(), ConcreteType::String);
        let redis_config_type = ConcreteType::Record(redis_config_fields);
        env.types
            .insert("RedisConfig".to_string(), redis_config_type);

        let mut redis_client_methods = HashMap::new();
        redis_client_methods.insert(
            "get".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Option(Box::new(ConcreteType::String))),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        redis_client_methods.insert(
            "set".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Bool),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        redis_client_methods.insert(
            "del".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Int),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        redis_client_methods.insert(
            "exists".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Bool),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        redis_client_methods.insert(
            "incr".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Int),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        redis_client_methods.insert(
            "decr".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Int),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        redis_client_methods.insert(
            "expire".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::Int],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Bool),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        redis_client_methods.insert(
            "ttl".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Int),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        redis_client_methods.insert(
            "keys".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::List(Box::new(ConcreteType::String))),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        redis_client_methods.insert(
            "hget".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::String],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Option(Box::new(ConcreteType::String))),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        redis_client_methods.insert(
            "hset".to_string(),
            ConcreteType::Function {
                params: vec![
                    ConcreteType::String,
                    ConcreteType::String,
                    ConcreteType::Any,
                ],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Int),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        redis_client_methods.insert(
            "hgetall".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Any),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        redis_client_methods.insert(
            "hdel".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String, ConcreteType::String],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Int),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        redis_client_methods.insert(
            "ping".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::String),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        redis_client_methods.insert(
            "flushall".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Bool),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        redis_client_methods.insert(
            "close".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Unit),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        redis_client_methods.insert(
            "quit".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(ConcreteType::Unit),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        let redis_client_type = ConcreteType::Record(redis_client_methods);
        env.types
            .insert("RedisClient".to_string(), redis_client_type.clone());

        let mut redis_module_methods = HashMap::new();
        redis_module_methods.insert(
            "createClient".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(redis_client_type.clone()),
            },
        );
        redis_module_methods.insert(
            "open".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(redis_client_type.clone()),
            },
        );
        redis_module_methods.insert(
            "connect".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::Any],
                return_type: Box::new(ConcreteType::Task(
                    Box::new(ConcreteType::Result(
                        Box::new(redis_client_type),
                        Box::new(ConcreteType::String),
                    )),
                    Box::new(ConcreteType::Any),
                )),
            },
        );
        let redis_module_type = ConcreteType::Record(redis_module_methods);
        env.types
            .insert("RedisModule".to_string(), redis_module_type.clone());
        env.variables.insert("redis".to_string(), redis_module_type);

        // Context & Cancellation Primitives (Go-style context.Context)
        let mut ctx_methods = HashMap::new();
        ctx_methods.insert(
            "done".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::RecvChannel(Box::new(ConcreteType::Unit))),
            },
        );
        ctx_methods.insert(
            "err".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Option(Box::new(ConcreteType::String))),
            },
        );
        ctx_methods.insert(
            "isDone".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(ConcreteType::Bool),
            },
        );
        ctx_methods.insert(
            "value".to_string(),
            ConcreteType::Function {
                params: vec![ConcreteType::String],
                return_type: Box::new(ConcreteType::Option(Box::new(ConcreteType::Any))),
            },
        );
        let context_inst_type = ConcreteType::Record(ctx_methods);
        env.types
            .insert("Context".to_string(), context_inst_type.clone());

        let mut ctx_static = HashMap::new();
        ctx_static.insert(
            "background".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(context_inst_type.clone()),
            },
        );
        ctx_static.insert(
            "todo".to_string(),
            ConcreteType::Function {
                params: vec![],
                return_type: Box::new(context_inst_type.clone()),
            },
        );
        ctx_static.insert(
            "withCancel".to_string(),
            ConcreteType::Function {
                params: vec![context_inst_type.clone()],
                return_type: Box::new(ConcreteType::Tuple(vec![
                    context_inst_type.clone(),
                    ConcreteType::Function {
                        params: vec![],
                        return_type: Box::new(ConcreteType::Unit),
                    },
                ])),
            },
        );
        ctx_static.insert(
            "withTimeout".to_string(),
            ConcreteType::Function {
                params: vec![context_inst_type.clone(), ConcreteType::Int],
                return_type: Box::new(ConcreteType::Tuple(vec![
                    context_inst_type.clone(),
                    ConcreteType::Function {
                        params: vec![],
                        return_type: Box::new(ConcreteType::Unit),
                    },
                ])),
            },
        );
        ctx_static.insert(
            "withValue".to_string(),
            ConcreteType::Function {
                params: vec![
                    context_inst_type.clone(),
                    ConcreteType::String,
                    ConcreteType::Any,
                ],
                return_type: Box::new(context_inst_type.clone()),
            },
        );
        env.variables
            .insert("Context".to_string(), ConcreteType::Record(ctx_static));

        env
    }

    pub fn enter_scope(&self) -> Self {
        Self {
            variables: HashMap::new(),
            types: HashMap::new(),
            parent: Some(Box::new(self.clone())),
        }
    }

    pub fn insert_var(&mut self, name: String, ty: ConcreteType) {
        self.variables.insert(name, ty);
    }

    pub fn lookup_var(&self, name: &str) -> Option<ConcreteType> {
        if let Some(ty) = self.variables.get(name) {
            Some(ty.clone())
        } else if let Some(ref parent) = self.parent {
            parent.lookup_var(name)
        } else {
            None
        }
    }

    pub fn insert_type(&mut self, name: String, ty: ConcreteType) {
        self.types.insert(name, ty);
    }

    pub fn lookup_type(&self, name: &str) -> Option<ConcreteType> {
        if let Some(ty) = self.types.get(name) {
            Some(ty.clone())
        } else if let Some(ref parent) = self.parent {
            parent.lookup_type(name)
        } else {
            None
        }
    }

    pub fn all_types(&self) -> HashMap<String, ConcreteType> {
        let mut all = if let Some(ref p) = self.parent {
            p.all_types()
        } else {
            HashMap::new()
        };
        all.extend(self.types.clone());
        all
    }

    pub fn all_variables(&self) -> HashMap<String, ConcreteType> {
        let mut all = if let Some(ref p) = self.parent {
            p.all_variables()
        } else {
            HashMap::new()
        };
        all.extend(self.variables.clone());
        all
    }
}

impl Clone for TypeEnv {
    fn clone(&self) -> Self {
        Self {
            variables: self.variables.clone(),
            types: self.types.clone(),
            parent: self.parent.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodSignature {
    pub name: String,
    pub receiver_name: String,
    pub receiver_type: ConcreteType,
    pub is_pointer: bool,
    pub params: Vec<ConcreteType>,
    pub return_type: ConcreteType,
}

pub struct TypeChecker {
    env: TypeEnv,
    pub methods_by_type: HashMap<String, Vec<MethodSignature>>,
    pub type_alias_map: HashMap<String, ConcreteType>,
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            env: TypeEnv::new(),
            methods_by_type: HashMap::new(),
            type_alias_map: HashMap::new(),
        }
    }

    pub fn env(&self) -> &TypeEnv {
        &self.env
    }

    pub fn env_mut(&mut self) -> &mut TypeEnv {
        &mut self.env
    }

    fn extract_base_type_name(&self, ast_type: &Type) -> Option<(String, bool)> {
        match ast_type {
            Type::Named { name, .. } => Some((name.clone(), false)),
            Type::Pointer(inner) => {
                if let Type::Named { name, .. } = &**inner {
                    Some((name.clone(), true))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn get_type_names_for_concrete_type(&self, ty: &ConcreteType) -> Vec<String> {
        let mut names = Vec::new();
        let target = match ty {
            ConcreteType::Pointer(inner) => &**inner,
            other => other,
        };
        for (name, alias_ty) in &self.type_alias_map {
            if alias_ty == target {
                names.push(name.clone());
            }
        }
        if let ConcreteType::SumType { name, .. } = target {
            names.push(name.clone());
        }
        names
    }

    pub fn lookup_method_for_type(&self, ty: &ConcreteType, member: &str) -> Option<ConcreteType> {
        let type_names = self.get_type_names_for_concrete_type(ty);
        for name in &type_names {
            if let Some(methods) = self.methods_by_type.get(name) {
                for m in methods {
                    if m.name == member {
                        return Some(ConcreteType::Function {
                            params: m.params.clone(),
                            return_type: Box::new(m.return_type.clone()),
                        });
                    }
                }
            }
        }
        None
    }

    fn find_variant_sum_type(&self, variant_name: &str) -> Option<ConcreteType> {
        for (_, ty) in self.env.all_types() {
            if let ConcreteType::SumType {
                ref name,
                ref variants,
                ref type_args,
            } = ty
            {
                if variants.contains_key(variant_name) {
                    return Some(ConcreteType::SumType {
                        name: name.clone(),
                        type_args: type_args.clone(),
                        variants: variants.clone(),
                    });
                }
            } else if let ConcreteType::Generic(_, ref inner) = ty
                && let ConcreteType::SumType { variants, .. } = inner.as_ref()
                && variants.contains_key(variant_name)
            {
                return Some(ty.clone());
            }
        }
        None
    }

    pub fn check_module(&mut self, module: &Module) -> Result<(), String> {
        // Pass 1: Register all sum types and type aliases
        for item in &module.items {
            match item {
                Item::SumType(sum) => {
                    let mut variants_map = HashMap::new();
                    for v in &sum.variants {
                        let shape = match &v.fields {
                            VariantFields::Unit => VariantFieldTypes::Unit,
                            VariantFields::Tuple(tys) => {
                                let resolved = tys
                                    .iter()
                                    .map(|t| self.resolve_ast_type(t))
                                    .collect::<Result<Vec<_>, _>>()?;
                                VariantFieldTypes::Tuple(resolved)
                            }
                            VariantFields::Record(fields) => {
                                let mut map = HashMap::new();
                                for (fname, ftype) in fields {
                                    map.insert(fname.clone(), self.resolve_ast_type(ftype)?);
                                }
                                VariantFieldTypes::Record(map)
                            }
                        };
                        variants_map.insert(v.name.clone(), shape);
                    }

                    let sum_type = ConcreteType::SumType {
                        name: sum.name.clone(),
                        type_args: vec![], // Base type has no args until instantiated
                        variants: variants_map,
                    };
                    let sum_type = if !sum.type_params.is_empty() {
                        ConcreteType::Generic(sum.type_params.clone(), Box::new(sum_type))
                    } else {
                        sum_type
                    };
                    self.env.insert_type(sum.name.clone(), sum_type);
                }
                Item::TypeAlias(alias) => {
                    let ty = self.resolve_ast_type(&alias.target)?;
                    let ty = if !alias.type_params.is_empty() {
                        ConcreteType::Generic(alias.type_params.clone(), Box::new(ty))
                    } else {
                        ty
                    };
                    self.env.insert_type(alias.name.clone(), ty.clone());
                    self.type_alias_map.insert(alias.name.clone(), ty);
                }
                Item::Interface(iface) => {
                    let mut methods = HashMap::new();
                    for m in &iface.methods {
                        let mut p_tys = Vec::new();
                        for p in &m.params {
                            p_tys.push(if let Some(ref ann) = p.type_annotation {
                                self.resolve_ast_type(ann)?
                            } else {
                                ConcreteType::Any
                            });
                        }
                        let r_ty = self.resolve_ast_type(&m.return_type)?;
                        methods.insert(m.name.clone(), (p_tys, r_ty));
                    }
                    let iface_ty = ConcreteType::Interface {
                        name: iface.name.clone(),
                        methods,
                    };
                    let iface_ty = if !iface.type_params.is_empty() {
                        ConcreteType::Generic(iface.type_params.clone(), Box::new(iface_ty))
                    } else {
                        iface_ty
                    };
                    self.env.insert_type(iface.name.clone(), iface_ty);
                }
                Item::Extern(ext) => {
                    for f in &ext.functions {
                        let param_types = f
                            .params
                            .iter()
                            .map(|p| {
                                p.type_annotation
                                    .as_ref()
                                    .map(|t| self.resolve_ast_type(t))
                                    .unwrap_or(Ok(ConcreteType::Any))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        let return_type = Box::new(self.resolve_ast_type(&f.return_type)?);
                        let mut func_ty = ConcreteType::Function {
                            params: param_types,
                            return_type,
                        };
                        if !f.type_params.is_empty() {
                            func_ty =
                                ConcreteType::Generic(f.type_params.clone(), Box::new(func_ty));
                        }
                        self.env.insert_var(f.name.clone(), func_ty);
                    }
                }
                _ => {}
            }
        }

        // Pass 2: Register function signatures
        for item in &module.items {
            if let Item::Function(func) = item {
                let param_types = func
                    .params
                    .iter()
                    .map(|p| {
                        if let Some(ref t) = p.type_annotation {
                            self.resolve_ast_type(t)
                        } else {
                            Ok(ConcreteType::TypeVar(p.name.clone()))
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                let ret_type = if let Some(ref t) = func.return_type {
                    self.resolve_ast_type(t)?
                } else {
                    ConcreteType::Unit
                };

                if let Some(ref recv) = func.receiver {
                    if let Some((base_name, is_ptr)) =
                        self.extract_base_type_name(&recv.target_type)
                    {
                        let recv_ty = self.resolve_ast_type(&recv.target_type)?;
                        let sig = MethodSignature {
                            name: func.name.clone(),
                            receiver_name: recv.name.clone(),
                            receiver_type: recv_ty,
                            is_pointer: is_ptr,
                            params: param_types,
                            return_type: ret_type,
                        };
                        self.methods_by_type.entry(base_name).or_default().push(sig);
                    }
                } else {
                    let mut func_ty = ConcreteType::Function {
                        params: param_types,
                        return_type: Box::new(ret_type),
                    };
                    if !func.type_params.is_empty() {
                        func_ty =
                            ConcreteType::Generic(func.type_params.clone(), Box::new(func_ty));
                    }
                    self.env.insert_var(func.name.clone(), func_ty);
                }
            }
        }

        // Pass 2.5: Process top-level statements so global let variables are in scope
        let mut module_env = self.env.clone();
        for item in &module.items {
            if let Item::Statement(stmt) = item {
                self.check_statement(stmt, &mut module_env)?;
            }
        }
        self.env = module_env;

        // Pass 3: Type check function bodies
        for item in &module.items {
            if let Item::Function(func) = item {
                let mut scope = self.env.enter_scope();
                if let Some(ref recv) = func.receiver {
                    let recv_ty = self.resolve_ast_type(&recv.target_type)?;
                    scope.insert_var(recv.name.clone(), recv_ty);
                }
                for p in &func.params {
                    let p_ty = if let Some(ref t) = p.type_annotation {
                        self.resolve_ast_type(t)?
                    } else {
                        ConcreteType::Any
                    };
                    scope.insert_var(p.name.clone(), p_ty);
                }

                let inferred_body_ty = self.infer_expr(&func.body, &mut scope)?;
                if let Some(ref declared_ret) = func.return_type {
                    let declared_ty = self.resolve_ast_type(declared_ret)?;
                    self.unify(
                        &declared_ty,
                        &inferred_body_ty,
                        &format!("Function '{}' return type", func.name),
                    )?;
                }
            }
        }

        Ok(())
    }

    pub fn make_iterator_type(&self, elem: ConcreteType) -> ConcreteType {
        let mut iter_methods = HashMap::new();
        iter_methods.insert(
            "next".to_string(),
            (vec![], ConcreteType::Option(Box::new(elem.clone()))),
        );
        iter_methods.insert(
            "map".to_string(),
            (
                vec![ConcreteType::Function {
                    params: vec![elem.clone()],
                    return_type: Box::new(ConcreteType::TypeVar("U".to_string())),
                }],
                ConcreteType::Interface {
                    name: "Iterator".to_string(),
                    methods: HashMap::new(),
                },
            ),
        );
        iter_methods.insert(
            "filter".to_string(),
            (
                vec![ConcreteType::Function {
                    params: vec![elem.clone()],
                    return_type: Box::new(ConcreteType::Bool),
                }],
                ConcreteType::Interface {
                    name: "Iterator".to_string(),
                    methods: HashMap::new(),
                },
            ),
        );
        iter_methods.insert(
            "sort".to_string(),
            (
                vec![],
                ConcreteType::Interface {
                    name: "Iterator".to_string(),
                    methods: HashMap::new(),
                },
            ),
        );
        iter_methods.insert(
            "toList".to_string(),
            (vec![], ConcreteType::List(Box::new(elem.clone()))),
        );
        iter_methods.insert(
            "toArray".to_string(),
            (vec![], ConcreteType::List(Box::new(elem))),
        );
        ConcreteType::Interface {
            name: "Iterator".to_string(),
            methods: iter_methods,
        }
    }

    pub fn is_iterator_type(&self, ty: &ConcreteType) -> Option<ConcreteType> {
        match ty {
            ConcreteType::Interface { name, methods } => {
                if name == "Iterator" || name == "Interator" || methods.contains_key("next") {
                    if let Some((_, ret)) = methods.get("next") {
                        match ret {
                            ConcreteType::Option(inner) => Some(*inner.clone()),
                            other => Some(other.clone()),
                        }
                    } else {
                        Some(ConcreteType::TypeVar("T".to_string()))
                    }
                } else {
                    None
                }
            }
            ConcreteType::Record(fields) => {
                if let Some(ConcreteType::Function { return_type, .. }) = fields.get("next") {
                    match &**return_type {
                        ConcreteType::Option(inner) => Some(*inner.clone()),
                        other => Some(other.clone()),
                    }
                } else {
                    None
                }
            }
            ConcreteType::Generic(_, inner) => self.is_iterator_type(inner),
            ConcreteType::TypeVar(name) if name == "Iterator" || name == "Interator" => {
                Some(ConcreteType::TypeVar("T".to_string()))
            }
            _ => None,
        }
    }

    pub fn resolve_ast_type(&self, ast_type: &Type) -> Result<ConcreteType, String> {
        match ast_type {
            Type::Named { name, type_args } => match name.as_str() {
                "Int" => Ok(ConcreteType::Int),
                "Float" => Ok(ConcreteType::Float),
                "String" => Ok(ConcreteType::String),
                "Bool" => Ok(ConcreteType::Bool),
                "Unit" => Ok(ConcreteType::Unit),
                "Any" => Ok(ConcreteType::Any),
                "Option" => {
                    let inner = type_args.first().ok_or("Option requires 1 type argument")?;
                    Ok(ConcreteType::Option(Box::new(
                        self.resolve_ast_type(inner)?,
                    )))
                }
                "Result" => {
                    let ok = type_args
                        .first()
                        .ok_or("Result requires 2 type arguments (Ok, Err)")?;
                    let err = type_args
                        .get(1)
                        .ok_or("Result requires 2 type arguments (Ok, Err)")?;
                    Ok(ConcreteType::Result(
                        Box::new(self.resolve_ast_type(ok)?),
                        Box::new(self.resolve_ast_type(err)?),
                    ))
                }
                "Task" => {
                    let ok = type_args
                        .first()
                        .ok_or("Task requires 2 type arguments (Ok, Err)")?;
                    let err = type_args
                        .get(1)
                        .ok_or("Task requires 2 type arguments (Ok, Err)")?;
                    Ok(ConcreteType::Task(
                        Box::new(self.resolve_ast_type(ok)?),
                        Box::new(self.resolve_ast_type(err)?),
                    ))
                }
                "List" | "Array" => {
                    let elem = if let Some(first) = type_args.first() {
                        self.resolve_ast_type(first)?
                    } else {
                        ConcreteType::Any
                    };
                    Ok(ConcreteType::List(Box::new(elem)))
                }
                "Channel" | "Chan" => {
                    let elem = type_args
                        .first()
                        .ok_or("Channel requires 1 type argument")?;
                    Ok(ConcreteType::Channel(Box::new(
                        self.resolve_ast_type(elem)?,
                    )))
                }
                "SendChannel" | "SendChan" => {
                    let elem = type_args
                        .first()
                        .ok_or("SendChannel requires 1 type argument")?;
                    Ok(ConcreteType::SendChannel(Box::new(
                        self.resolve_ast_type(elem)?,
                    )))
                }
                "RecvChannel" | "RecvChan" => {
                    let elem = type_args
                        .first()
                        .ok_or("RecvChannel requires 1 type argument")?;
                    Ok(ConcreteType::RecvChannel(Box::new(
                        self.resolve_ast_type(elem)?,
                    )))
                }
                custom => {
                    let resolved_args: Result<Vec<ConcreteType>, String> =
                        type_args.iter().map(|t| self.resolve_ast_type(t)).collect();
                    let resolved_args = resolved_args?;
                    if let Some(ty) = self.env.lookup_type(custom) {
                        if let ConcreteType::Generic(params, inner) = ty {
                            if params.len() == resolved_args.len() {
                                let mut bindings = std::collections::HashMap::new();
                                for (p, a) in params.iter().zip(resolved_args.iter()) {
                                    bindings.insert(p.clone(), a.clone());
                                }
                                let mut inst = inner.substitute(&bindings);
                                if let ConcreteType::SumType {
                                    ref mut type_args, ..
                                } = inst
                                {
                                    *type_args = resolved_args.clone();
                                }
                                Ok(inst)
                            } else if resolved_args.is_empty() {
                                Ok(ConcreteType::Generic(params, inner))
                            } else {
                                Err(format!(
                                    "Generic type '{}' expects {} type arguments, but {} were provided",
                                    custom,
                                    params.len(),
                                    resolved_args.len()
                                ))
                            }
                        } else {
                            if !resolved_args.is_empty() {
                                Err(format!(
                                    "Type '{}' is not generic, but type arguments were provided",
                                    custom
                                ))
                            } else {
                                Ok(ty)
                            }
                        }
                    } else {
                        Ok(ConcreteType::TypeVar(custom.to_string()))
                    }
                }
            },
            Type::Record(fields) => {
                let mut map = HashMap::new();
                for (k, v) in fields {
                    let clean_k = k.trim_end_matches('?').to_string();
                    map.insert(clean_k, self.resolve_ast_type(v)?);
                }
                Ok(ConcreteType::Record(map))
            }
            Type::Tuple(types) => {
                let tys = types
                    .iter()
                    .map(|t| self.resolve_ast_type(t))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ConcreteType::Tuple(tys))
            }
            Type::Function {
                params,
                return_type,
            } => {
                let p = params
                    .iter()
                    .map(|t| self.resolve_ast_type(t))
                    .collect::<Result<Vec<_>, _>>()?;
                let ret = self.resolve_ast_type(return_type)?;
                Ok(ConcreteType::Function {
                    params: p,
                    return_type: Box::new(ret),
                })
            }
            Type::Pointer(inner) => Ok(ConcreteType::Pointer(Box::new(
                self.resolve_ast_type(inner)?,
            ))),
            Type::SendChannel(inner) => Ok(ConcreteType::SendChannel(Box::new(
                self.resolve_ast_type(inner)?,
            ))),
            Type::RecvChannel(inner) => Ok(ConcreteType::RecvChannel(Box::new(
                self.resolve_ast_type(inner)?,
            ))),
            Type::Unit => Ok(ConcreteType::Unit),
            Type::TypeVar(v) => Ok(ConcreteType::TypeVar(v.clone())),
        }
    }

    fn check_statement(&mut self, stmt: &Statement, env: &mut TypeEnv) -> Result<(), String> {
        match stmt {
            Statement::Let {
                name,
                type_annotation,
                value,
                ..
            } => {
                let val_ty = self.infer_expr(value, env)?;
                if let Some(annot) = type_annotation {
                    let annot_ty = self.resolve_ast_type(annot)?;
                    self.unify(&annot_ty, &val_ty, &format!("Let binding '{}'", name))?;
                    env.insert_var(name.clone(), annot_ty);
                } else {
                    env.insert_var(name.clone(), val_ty);
                }
                Ok(())
            }
            Statement::LetPattern {
                pattern,
                type_annotation,
                value,
            } => {
                let val_ty = self.infer_expr(value, env)?;
                if let Some(annot) = type_annotation {
                    let annot_ty = self.resolve_ast_type(annot)?;
                    self.unify(&annot_ty, &val_ty, "Let pattern binding")?;
                    self.bind_pattern_with_type(pattern, &annot_ty, env)?;
                } else {
                    self.bind_pattern_with_type(pattern, &val_ty, env)?;
                }
                Ok(())
            }
            Statement::Assign { target, value } => {
                let target_ty = self.infer_expr(target, env)?;
                let val_ty = self.infer_expr(value, env)?;
                self.unify(&target_ty, &val_ty, "Variable reassignment")?;
                Ok(())
            }
            Statement::Defer(expr) | Statement::ErrDefer(expr) => {
                self.infer_expr(expr, env)?;
                Ok(())
            }
            Statement::Expr(e) => {
                self.infer_expr(e, env)?;
                Ok(())
            }
            Statement::Return(opt_expr) => {
                if let Some(e) = opt_expr {
                    self.infer_expr(e, env)?;
                }
                Ok(())
            }
        }
    }

    pub fn infer_expr(&mut self, expr: &Expr, env: &mut TypeEnv) -> Result<ConcreteType, String> {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::Int(_) => Ok(ConcreteType::Int),
                Literal::Float(_) => Ok(ConcreteType::Float),
                Literal::String(_) => Ok(ConcreteType::String),
                Literal::TemplateString(_) => Ok(ConcreteType::String),
                Literal::Bool(_) => Ok(ConcreteType::Bool),
                Literal::Unit => Ok(ConcreteType::Unit),
            },
            Expr::Identifier(id) => {
                if let Some(ty) = env.lookup_var(id) {
                    Ok(ty)
                } else if let Some(sum_ty) = env.lookup_type(id) {
                    Ok(sum_ty)
                } else if let Some(sum_ty) = self.find_variant_sum_type(id) {
                    if let ConcreteType::Generic(type_params, inner) = sum_ty {
                        if let ConcreteType::SumType {
                            name,
                            variants,
                            type_args,
                        } = inner.as_ref()
                        {
                            let mut t_args = type_args.clone();
                            if t_args.is_empty() {
                                t_args =
                                    type_params.into_iter().map(ConcreteType::TypeVar).collect();
                            }
                            Ok(ConcreteType::SumType {
                                name: name.clone(),
                                variants: variants.clone(),
                                type_args: t_args,
                            })
                        } else {
                            Ok(inner.as_ref().clone())
                        }
                    } else {
                        Ok(sum_ty)
                    }
                } else if id == "Some" {
                    Ok(ConcreteType::Function {
                        params: vec![ConcreteType::TypeVar("T".to_string())],
                        return_type: Box::new(ConcreteType::Option(Box::new(
                            ConcreteType::TypeVar("T".to_string()),
                        ))),
                    })
                } else if id == "None" {
                    Ok(ConcreteType::Option(Box::new(ConcreteType::TypeVar(
                        "T".to_string(),
                    ))))
                } else if id == "Ok" {
                    Ok(ConcreteType::Function {
                        params: vec![ConcreteType::TypeVar("T".to_string())],
                        return_type: Box::new(ConcreteType::Result(
                            Box::new(ConcreteType::TypeVar("T".to_string())),
                            Box::new(ConcreteType::TypeVar("E".to_string())),
                        )),
                    })
                } else if id == "Err" {
                    Ok(ConcreteType::Function {
                        params: vec![ConcreteType::TypeVar("E".to_string())],
                        return_type: Box::new(ConcreteType::Result(
                            Box::new(ConcreteType::TypeVar("T".to_string())),
                            Box::new(ConcreteType::TypeVar("E".to_string())),
                        )),
                    })
                } else if let Some(ty) = env.lookup_type(id) {
                    Ok(ty)
                } else {
                    Err(format!("Undefined variable '{}'", id))
                }
            }
            Expr::Binary { op, left, right } => {
                let left_ty = self.infer_expr(left, env)?;
                let right_ty = self.infer_expr(right, env)?;

                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        if (left_ty == ConcreteType::Float && right_ty == ConcreteType::Int)
                            || (left_ty == ConcreteType::Int && right_ty == ConcreteType::Float)
                        {
                            Ok(ConcreteType::Float)
                        } else {
                            self.unify(&left_ty, &right_ty, "Binary arithmetic operands")?;
                            Ok(left_ty)
                        }
                    }
                    BinOp::Equal
                    | BinOp::NotEqual
                    | BinOp::LessThan
                    | BinOp::LessEqual
                    | BinOp::GreaterThan
                    | BinOp::GreaterEqual => {
                        if (left_ty == ConcreteType::Float && right_ty == ConcreteType::Int)
                            || (left_ty == ConcreteType::Int && right_ty == ConcreteType::Float)
                        {
                            Ok(ConcreteType::Bool)
                        } else {
                            self.unify(&left_ty, &right_ty, "Comparison operands")?;
                            Ok(ConcreteType::Bool)
                        }
                    }
                    BinOp::And | BinOp::Or => {
                        self.unify(&left_ty, &ConcreteType::Bool, "Logical left operand")?;
                        self.unify(&right_ty, &ConcreteType::Bool, "Logical right operand")?;
                        Ok(ConcreteType::Bool)
                    }
                }
            }
            Expr::Unary { op, expr } => {
                let inner = self.infer_expr(expr, env)?;
                match op {
                    UnOp::Negate => {
                        if inner == ConcreteType::Int || inner == ConcreteType::Float {
                            Ok(inner)
                        } else {
                            Err(format!(
                                "Negation '-' requires numeric type, found {}",
                                inner
                            ))
                        }
                    }
                    UnOp::Not => {
                        self.unify(&inner, &ConcreteType::Bool, "Logical negation '!'")?;
                        Ok(ConcreteType::Bool)
                    }
                }
            }
            Expr::Pipeline { left, right } => match &**right {
                Expr::FunctionCall { callee, args } => {
                    let mut new_args = Vec::new();
                    let has_placeholder = args.iter().any(|a| matches!(a, Expr::Placeholder));
                    if has_placeholder {
                        for a in args {
                            if matches!(a, Expr::Placeholder) {
                                new_args.push(*left.clone());
                            } else {
                                new_args.push(a.clone());
                            }
                        }
                    } else {
                        new_args.push(*left.clone());
                        new_args.extend(args.clone());
                    }
                    self.infer_expr(
                        &Expr::FunctionCall {
                            callee: callee.clone(),
                            args: new_args,
                        },
                        env,
                    )
                }
                _ => {
                    let val_ty = self.infer_expr(left, env)?;
                    let func_ty = self.infer_expr(right, env)?;

                    match func_ty {
                        ConcreteType::Function {
                            params,
                            return_type,
                        } => {
                            if let Some(first_param) = params.first() {
                                self.unify(first_param, &val_ty, "Pipeline argument")?;
                                Ok(*return_type)
                            } else {
                                Err("Cannot pipe value into zero-argument function".to_string())
                            }
                        }
                        ConcreteType::Any => Ok(ConcreteType::Any),
                        other => Err(format!(
                            "Right side of pipeline '|>' must be a function, found {}",
                            other
                        )),
                    }
                }
            },
            Expr::FunctionCall { callee, args } => {
                // Check if callee is MemberAccess for map, filter, sort, toList, toArray
                if let Expr::MemberAccess { object, member } = &**callee {
                    let obj_ty = self.infer_expr(object, env)?;
                    if member == "map" {
                        if let Some(arg) = args.first() {
                            let arg_ty = self.infer_expr(arg, env)?;
                            let ret_ty = match arg_ty {
                                ConcreteType::Function { return_type, .. } => *return_type,
                                _ => ConcreteType::Any,
                            };
                            if self.is_iterator_type(&obj_ty).is_some() {
                                return Ok(self.make_iterator_type(ret_ty));
                            } else {
                                return Ok(ConcreteType::List(Box::new(ret_ty)));
                            }
                        }
                    } else if member == "filter" || member == "sort" {
                        if let Some(arg) = args.first() {
                            let _ = self.infer_expr(arg, env)?;
                        }
                        return Ok(obj_ty);
                    } else if member == "toList" || member == "toArray" {
                        let elem = self.is_iterator_type(&obj_ty).unwrap_or(ConcreteType::Any);
                        return Ok(ConcreteType::List(Box::new(elem)));
                    } else if member == "slice" {
                        for arg in args {
                            let _ = self.infer_expr(arg, env)?;
                        }
                        return Ok(obj_ty);
                    }
                }

                // Check if callee is Identifier for map, filter, sort, iter
                if let Expr::Identifier(fn_name) = &**callee {
                    if (fn_name == "len" || fn_name == "cap") && !args.is_empty() {
                        let _ = self.infer_expr(&args[0], env)?;
                        return Ok(ConcreteType::Int);
                    }
                    if fn_name == "append" && !args.is_empty() {
                        let coll_ty = self.infer_expr(&args[0], env)?;
                        for arg in &args[1..] {
                            let _ = self.infer_expr(arg, env)?;
                        }
                        return Ok(coll_ty);
                    }
                    if fn_name == "make" && !args.is_empty() {
                        let ty = self.infer_expr(&args[0], env)?;
                        return Ok(ty);
                    }
                    if fn_name == "map" && args.len() >= 2 {
                        let coll_ty = self.infer_expr(&args[0], env)?;
                        let arg_ty = self.infer_expr(&args[1], env)?;
                        let ret_ty = match arg_ty {
                            ConcreteType::Function { return_type, .. } => *return_type,
                            _ => ConcreteType::Any,
                        };
                        if self.is_iterator_type(&coll_ty).is_some() {
                            return Ok(self.make_iterator_type(ret_ty));
                        } else {
                            return Ok(ConcreteType::List(Box::new(ret_ty)));
                        }
                    } else if fn_name == "filter" && args.len() >= 2 {
                        let coll_ty = self.infer_expr(&args[0], env)?;
                        let _ = self.infer_expr(&args[1], env)?;
                        return Ok(coll_ty);
                    } else if fn_name == "sort" && !args.is_empty() {
                        let coll_ty = self.infer_expr(&args[0], env)?;
                        if args.len() >= 2 {
                            let _ = self.infer_expr(&args[1], env)?;
                        }
                        return Ok(coll_ty);
                    } else if fn_name == "iter" && !args.is_empty() {
                        let coll_ty = self.infer_expr(&args[0], env)?;
                        let elem = match coll_ty {
                            ConcreteType::List(e) => *e,
                            _ => self.is_iterator_type(&coll_ty).unwrap_or(ConcreteType::Any),
                        };
                        return Ok(self.make_iterator_type(elem));
                    } else if (fn_name == "toList" || fn_name == "toArray") && !args.is_empty() {
                        let coll_ty = self.infer_expr(&args[0], env)?;
                        let elem = match coll_ty {
                            ConcreteType::List(e) => *e,
                            _ => self.is_iterator_type(&coll_ty).unwrap_or(ConcreteType::Any),
                        };
                        return Ok(ConcreteType::List(Box::new(elem)));
                    }
                }

                let callee_ty = self.infer_expr(callee, env)?;
                match callee_ty {
                    ConcreteType::Function {
                        params,
                        return_type,
                    } => {
                        for (arg, param) in args.iter().zip(params.iter()) {
                            let arg_ty = self.infer_expr(arg, env)?;
                            self.unify(param, &arg_ty, "Function argument")?;
                        }
                        Ok(*return_type)
                    }
                    ConcreteType::SumType {
                        name,
                        type_args,
                        variants,
                    } => {
                        // Constructor call for sum type variant with record or tuple payload
                        Ok(ConcreteType::SumType {
                            name,
                            type_args,
                            variants,
                        })
                    }
                    ConcreteType::Generic(type_params, inner) => {
                        match *inner {
                            ConcreteType::Function {
                                params,
                                return_type,
                            } => {
                                // For simplicity, we don't fully instantiate fresh variables here yet,
                                // we just rely on unify to handle TypeVars.
                                for (arg, param) in args.iter().zip(params.iter()) {
                                    let arg_ty = self.infer_expr(arg, env)?;
                                    self.unify(param, &arg_ty, "Function argument")?;
                                }
                                Ok(*return_type)
                            }
                            ConcreteType::SumType {
                                name,
                                mut type_args,
                                variants,
                            } => {
                                // If the constructor is called, we can try to infer type_args from arguments
                                // For now, just return the SumType with TypeVars and let unify handle it
                                if type_args.is_empty() {
                                    type_args = type_params
                                        .into_iter()
                                        .map(ConcreteType::TypeVar)
                                        .collect();
                                }
                                Ok(ConcreteType::SumType {
                                    name,
                                    type_args,
                                    variants,
                                })
                            }
                            _ => Err(format!(
                                "Attempted to call non-callable generic type {}",
                                inner
                            )),
                        }
                    }
                    ConcreteType::Any => Ok(ConcreteType::Any),
                    other => Err(format!("Attempted to call non-function of type {}", other)),
                }
            }

            Expr::MemberAccess { object, member } => {
                let obj_ty = self.infer_expr(object, env)?;
                match obj_ty {
                    ConcreteType::Record(fields) => {
                        if let Some(field_ty) = fields.get(member) {
                            Ok(field_ty.clone())
                        } else if let Some(method_ty) = self
                            .lookup_method_for_type(&ConcreteType::Record(fields.clone()), member)
                        {
                            Ok(method_ty)
                        } else if let Some(elem) =
                            self.is_iterator_type(&ConcreteType::Record(fields.clone()))
                        {
                            match member.as_str() {
                                "next" => Ok(ConcreteType::Function {
                                    params: vec![],
                                    return_type: Box::new(ConcreteType::Option(Box::new(elem))),
                                }),
                                "map" => Ok(ConcreteType::Function {
                                    params: vec![ConcreteType::Function {
                                        params: vec![elem.clone()],
                                        return_type: Box::new(ConcreteType::TypeVar(
                                            "U".to_string(),
                                        )),
                                    }],
                                    return_type: Box::new(self.make_iterator_type(
                                        ConcreteType::TypeVar("U".to_string()),
                                    )),
                                }),
                                "filter" => Ok(ConcreteType::Function {
                                    params: vec![ConcreteType::Function {
                                        params: vec![elem.clone()],
                                        return_type: Box::new(ConcreteType::Bool),
                                    }],
                                    return_type: Box::new(self.make_iterator_type(elem)),
                                }),
                                "sort" => Ok(ConcreteType::Function {
                                    params: vec![ConcreteType::Function {
                                        params: vec![elem.clone(), elem.clone()],
                                        return_type: Box::new(ConcreteType::Int),
                                    }],
                                    return_type: Box::new(self.make_iterator_type(elem)),
                                }),
                                "toList" | "toArray" => Ok(ConcreteType::Function {
                                    params: vec![],
                                    return_type: Box::new(ConcreteType::List(Box::new(elem))),
                                }),
                                _ => Err(format!("Field '{}' not found in record", member)),
                            }
                        } else {
                            Err(format!("Field '{}' not found in record", member))
                        }
                    }
                    ConcreteType::Interface {
                        ref name,
                        ref methods,
                    } => {
                        if let Some((params, ret)) = methods.get(member) {
                            Ok(ConcreteType::Function {
                                params: params.clone(),
                                return_type: Box::new(ret.clone()),
                            })
                        } else if name == "Iterator"
                            || name == "Interator"
                            || methods.contains_key("next")
                        {
                            let elem = self
                                .is_iterator_type(&ConcreteType::Interface {
                                    name: name.clone(),
                                    methods: methods.clone(),
                                })
                                .unwrap_or(ConcreteType::Any);
                            match member.as_str() {
                                "map" => Ok(ConcreteType::Function {
                                    params: vec![ConcreteType::Function {
                                        params: vec![elem.clone()],
                                        return_type: Box::new(ConcreteType::TypeVar(
                                            "U".to_string(),
                                        )),
                                    }],
                                    return_type: Box::new(self.make_iterator_type(
                                        ConcreteType::TypeVar("U".to_string()),
                                    )),
                                }),
                                "filter" => Ok(ConcreteType::Function {
                                    params: vec![ConcreteType::Function {
                                        params: vec![elem.clone()],
                                        return_type: Box::new(ConcreteType::Bool),
                                    }],
                                    return_type: Box::new(self.make_iterator_type(elem)),
                                }),
                                "sort" => Ok(ConcreteType::Function {
                                    params: vec![ConcreteType::Function {
                                        params: vec![elem.clone(), elem.clone()],
                                        return_type: Box::new(ConcreteType::Int),
                                    }],
                                    return_type: Box::new(self.make_iterator_type(elem)),
                                }),
                                "toList" | "toArray" => Ok(ConcreteType::Function {
                                    params: vec![],
                                    return_type: Box::new(ConcreteType::List(Box::new(elem))),
                                }),
                                _ => Err(format!(
                                    "Method '{}' not found in interface '{}'",
                                    member, name
                                )),
                            }
                        } else {
                            Err(format!(
                                "Method '{}' not found in interface '{}'",
                                member, name
                            ))
                        }
                    }
                    ConcreteType::Pointer(ref inner) => {
                        if let Some(method_ty) = self.lookup_method_for_type(&obj_ty, member) {
                            Ok(method_ty)
                        } else if let Some(method_ty) = self.lookup_method_for_type(inner, member) {
                            Ok(method_ty)
                        } else if let ConcreteType::Record(fields) = &**inner {
                            if let Some(field_ty) = fields.get(member) {
                                Ok(field_ty.clone())
                            } else if member == "val" || member == "value" {
                                Ok(*inner.clone())
                            } else {
                                Err(format!("Field '{}' not found in record", member))
                            }
                        } else if member == "val" || member == "value" {
                            Ok(*inner.clone())
                        } else {
                            Err(format!("Unknown property '{}' on Pointer", member))
                        }
                    }
                    ConcreteType::Channel(ref inner) => match member.as_str() {
                        "send" => Ok(ConcreteType::Function {
                            params: vec![*inner.clone()],
                            return_type: Box::new(ConcreteType::Bool),
                        }),
                        "recv" => Ok(ConcreteType::Function {
                            params: vec![],
                            return_type: Box::new(ConcreteType::Option(inner.clone())),
                        }),
                        "close" => Ok(ConcreteType::Function {
                            params: vec![],
                            return_type: Box::new(ConcreteType::Unit),
                        }),
                        _ => Err(format!("Unknown method '{}' on Channel", member)),
                    },
                    ConcreteType::SumType {
                        name,
                        type_args,
                        variants,
                    } => {
                        if variants.contains_key(member) {
                            Ok(ConcreteType::SumType {
                                name,
                                type_args,
                                variants,
                            })
                        } else {
                            Err(format!("Variant '{}' not found in type '{}'", member, name))
                        }
                    }
                    ConcreteType::TypeVar(ref type_name) => {
                        if type_name == "Iterator" || type_name == "Interator" {
                            let elem = ConcreteType::TypeVar("T".to_string());
                            match member.as_str() {
                                "next" => Ok(ConcreteType::Function {
                                    params: vec![],
                                    return_type: Box::new(ConcreteType::Option(Box::new(elem))),
                                }),
                                "map" => Ok(ConcreteType::Function {
                                    params: vec![ConcreteType::Function {
                                        params: vec![elem.clone()],
                                        return_type: Box::new(ConcreteType::TypeVar(
                                            "U".to_string(),
                                        )),
                                    }],
                                    return_type: Box::new(self.make_iterator_type(
                                        ConcreteType::TypeVar("U".to_string()),
                                    )),
                                }),
                                "filter" => Ok(ConcreteType::Function {
                                    params: vec![ConcreteType::Function {
                                        params: vec![elem.clone()],
                                        return_type: Box::new(ConcreteType::Bool),
                                    }],
                                    return_type: Box::new(self.make_iterator_type(elem)),
                                }),
                                "sort" => Ok(ConcreteType::Function {
                                    params: vec![ConcreteType::Function {
                                        params: vec![elem.clone(), elem.clone()],
                                        return_type: Box::new(ConcreteType::Int),
                                    }],
                                    return_type: Box::new(self.make_iterator_type(elem)),
                                }),
                                "toList" | "toArray" => Ok(ConcreteType::Function {
                                    params: vec![],
                                    return_type: Box::new(ConcreteType::List(Box::new(elem))),
                                }),
                                _ => Ok(ConcreteType::Any),
                            }
                        } else if let Some(ConcreteType::SumType {
                            name,
                            type_args,
                            variants,
                        }) = env.lookup_type(type_name)
                        {
                            if variants.contains_key(member) {
                                Ok(ConcreteType::SumType {
                                    name,
                                    type_args,
                                    variants,
                                })
                            } else {
                                Err(format!("Variant '{}' not found in type '{}'", member, name))
                            }
                        } else if let Some(ConcreteType::Interface { name, methods }) =
                            env.lookup_type(type_name)
                        {
                            if let Some((params, ret)) = methods.get(member) {
                                Ok(ConcreteType::Function {
                                    params: params.clone(),
                                    return_type: Box::new(ret.clone()),
                                })
                            } else {
                                Err(format!(
                                    "Method '{}' not found in interface '{}'",
                                    member, name
                                ))
                            }
                        } else if let Some(ConcreteType::Record(fields)) =
                            env.lookup_type(type_name)
                        {
                            if let Some(field_ty) = fields.get(member) {
                                Ok(field_ty.clone())
                            } else {
                                Err(format!("Field '{}' not found in record", member))
                            }
                        } else {
                            Ok(ConcreteType::Any)
                        }
                    }
                    ConcreteType::Generic(_, ref inner) => match &**inner {
                        ConcreteType::Interface { name, methods } => {
                            if let Some((params, ret)) = methods.get(member) {
                                Ok(ConcreteType::Function {
                                    params: params.clone(),
                                    return_type: Box::new(ret.clone()),
                                })
                            } else if name == "Iterator"
                                || name == "Interator"
                                || methods.contains_key("next")
                            {
                                let elem =
                                    self.is_iterator_type(inner).unwrap_or(ConcreteType::Any);
                                match member.as_str() {
                                    "map" => Ok(ConcreteType::Function {
                                        params: vec![ConcreteType::Function {
                                            params: vec![elem.clone()],
                                            return_type: Box::new(ConcreteType::TypeVar(
                                                "U".to_string(),
                                            )),
                                        }],
                                        return_type: Box::new(self.make_iterator_type(
                                            ConcreteType::TypeVar("U".to_string()),
                                        )),
                                    }),
                                    "filter" => Ok(ConcreteType::Function {
                                        params: vec![ConcreteType::Function {
                                            params: vec![elem.clone()],
                                            return_type: Box::new(ConcreteType::Bool),
                                        }],
                                        return_type: Box::new(self.make_iterator_type(elem)),
                                    }),
                                    "sort" => Ok(ConcreteType::Function {
                                        params: vec![ConcreteType::Function {
                                            params: vec![elem.clone(), elem.clone()],
                                            return_type: Box::new(ConcreteType::Int),
                                        }],
                                        return_type: Box::new(self.make_iterator_type(elem)),
                                    }),
                                    "toList" | "toArray" => Ok(ConcreteType::Function {
                                        params: vec![],
                                        return_type: Box::new(ConcreteType::List(Box::new(elem))),
                                    }),
                                    _ => Err(format!(
                                        "Method '{}' not found in interface '{}'",
                                        member, name
                                    )),
                                }
                            } else {
                                Err(format!(
                                    "Method '{}' not found in interface '{}'",
                                    member, name
                                ))
                            }
                        }
                        ConcreteType::Record(fields) => {
                            if let Some(field_ty) = fields.get(member) {
                                Ok(field_ty.clone())
                            } else {
                                Err(format!("Field '{}' not found in record", member))
                            }
                        }
                        _ => Ok(ConcreteType::Any),
                    },
                    ConcreteType::String => match member.as_str() {
                        "length" => Ok(ConcreteType::Int),
                        "trim" | "trimStart" | "trimEnd" | "toLowerCase" | "toUpperCase" => {
                            Ok(ConcreteType::Function {
                                params: vec![],
                                return_type: Box::new(ConcreteType::String),
                            })
                        }
                        "split" => Ok(ConcreteType::Function {
                            params: vec![ConcreteType::String],
                            return_type: Box::new(ConcreteType::List(Box::new(
                                ConcreteType::String,
                            ))),
                        }),
                        "includes" | "startsWith" | "endsWith" => Ok(ConcreteType::Function {
                            params: vec![ConcreteType::String],
                            return_type: Box::new(ConcreteType::Bool),
                        }),
                        "replace" | "replaceAll" => Ok(ConcreteType::Function {
                            params: vec![ConcreteType::String, ConcreteType::String],
                            return_type: Box::new(ConcreteType::String),
                        }),
                        "charAt" => Ok(ConcreteType::Function {
                            params: vec![ConcreteType::Int],
                            return_type: Box::new(ConcreteType::String),
                        }),
                        "charCodeAt" => Ok(ConcreteType::Function {
                            params: vec![ConcreteType::Int],
                            return_type: Box::new(ConcreteType::Int),
                        }),
                        "substring" | "slice" => Ok(ConcreteType::Function {
                            params: vec![ConcreteType::Int, ConcreteType::Int],
                            return_type: Box::new(ConcreteType::String),
                        }),
                        "indexOf" => Ok(ConcreteType::Function {
                            params: vec![ConcreteType::String],
                            return_type: Box::new(ConcreteType::Int),
                        }),
                        _ => Ok(ConcreteType::Any),
                    },
                    ConcreteType::List(ref elem) => match member.as_str() {
                        "length" | "len" => Ok(ConcreteType::Int),
                        "capacity" | "cap" => Ok(ConcreteType::Int),
                        "push" => Ok(ConcreteType::Function {
                            params: vec![*elem.clone()],
                            return_type: Box::new(ConcreteType::Int),
                        }),
                        "pop" => Ok(ConcreteType::Function {
                            params: vec![],
                            return_type: Box::new(ConcreteType::Option(elem.clone())),
                        }),
                        "join" => Ok(ConcreteType::Function {
                            params: vec![ConcreteType::String],
                            return_type: Box::new(ConcreteType::String),
                        }),
                        "includes" => Ok(ConcreteType::Function {
                            params: vec![*elem.clone()],
                            return_type: Box::new(ConcreteType::Bool),
                        }),
                        "slice" | "concat" | "reverse" => Ok(ConcreteType::Function {
                            params: vec![],
                            return_type: Box::new(ConcreteType::List(elem.clone())),
                        }),
                        "map" => Ok(ConcreteType::Function {
                            params: vec![ConcreteType::Function {
                                params: vec![*elem.clone()],
                                return_type: Box::new(ConcreteType::TypeVar("U".to_string())),
                            }],
                            return_type: Box::new(ConcreteType::List(Box::new(
                                ConcreteType::TypeVar("U".to_string()),
                            ))),
                        }),
                        "filter" => Ok(ConcreteType::Function {
                            params: vec![ConcreteType::Function {
                                params: vec![*elem.clone()],
                                return_type: Box::new(ConcreteType::Bool),
                            }],
                            return_type: Box::new(ConcreteType::List(elem.clone())),
                        }),
                        "sort" => Ok(ConcreteType::Function {
                            params: vec![ConcreteType::Function {
                                params: vec![*elem.clone(), *elem.clone()],
                                return_type: Box::new(ConcreteType::Int),
                            }],
                            return_type: Box::new(ConcreteType::List(elem.clone())),
                        }),
                        _ => Ok(ConcreteType::Any),
                    },
                    ConcreteType::Map(ref k, ref v) => match member.as_str() {
                        "size" => Ok(ConcreteType::Int),
                        "get" => Ok(ConcreteType::Function {
                            params: vec![*k.clone()],
                            return_type: Box::new(ConcreteType::Option(v.clone())),
                        }),
                        "set" => Ok(ConcreteType::Function {
                            params: vec![*k.clone(), *v.clone()],
                            return_type: Box::new(ConcreteType::Unit),
                        }),
                        "has" | "delete" => Ok(ConcreteType::Function {
                            params: vec![*k.clone()],
                            return_type: Box::new(ConcreteType::Bool),
                        }),
                        "clear" => Ok(ConcreteType::Function {
                            params: vec![],
                            return_type: Box::new(ConcreteType::Unit),
                        }),
                        _ => Ok(ConcreteType::Any),
                    },
                    ConcreteType::Any => Ok(ConcreteType::Any),
                    other => Err(format!("Cannot access property on type {}", other)),
                }
            }
            Expr::IndexAccess { object, index } => {
                let _idx_ty = self.infer_expr(index, env)?;
                let obj_ty = self.infer_expr(object, env)?;
                match obj_ty {
                    ConcreteType::List(elem) => Ok(*elem),
                    ConcreteType::String => Ok(ConcreteType::String),
                    ConcreteType::Any => Ok(ConcreteType::Any),
                    other => Err(format!("Cannot index into type {}", other)),
                }
            }
            Expr::SliceAccess {
                object,
                low,
                high,
                max,
            } => {
                if let Some(l) = low {
                    let l_ty = self.infer_expr(l, env)?;
                    self.unify(&ConcreteType::Int, &l_ty, "Slice lower bound must be Int")?;
                }
                if let Some(h) = high {
                    let h_ty = self.infer_expr(h, env)?;
                    self.unify(&ConcreteType::Int, &h_ty, "Slice upper bound must be Int")?;
                }
                if let Some(m) = max {
                    let m_ty = self.infer_expr(m, env)?;
                    self.unify(&ConcreteType::Int, &m_ty, "Slice max bound must be Int")?;
                }
                let obj_ty = self.infer_expr(object, env)?;
                match &obj_ty {
                    ConcreteType::List(_) | ConcreteType::String | ConcreteType::Any => Ok(obj_ty),
                    other => Err(format!("Cannot slice type {}", other)),
                }
            }
            Expr::RecordLiteral { fields, spread } => {
                let mut map = HashMap::new();
                if let Some(sp) = spread {
                    let sp_ty = self.infer_expr(sp, env)?;
                    match sp_ty {
                        ConcreteType::Record(sp_fields) => {
                            map.extend(sp_fields);
                        }
                        ConcreteType::TypeVar(ref type_name) => {
                            if let Some(ConcreteType::Record(sp_fields)) =
                                env.lookup_type(type_name)
                            {
                                map.extend(sp_fields);
                            }
                        }
                        _ => {}
                    }
                }
                for (name, val) in fields {
                    let val_ty = self.infer_expr(val, env)?;
                    map.insert(name.clone(), val_ty);
                }
                Ok(ConcreteType::Record(map))
            }
            Expr::ListLiteral(items) => {
                if let Some(first) = items.first() {
                    let mut elem_ty = self.infer_expr(first, env)?;
                    for item in items.iter().skip(1) {
                        let it_ty = self.infer_expr(item, env)?;
                        if self
                            .unify(&elem_ty, &it_ty, "List element consistency")
                            .is_err()
                        {
                            elem_ty = ConcreteType::Any;
                        }
                    }
                    Ok(ConcreteType::List(Box::new(elem_ty)))
                } else {
                    Ok(ConcreteType::List(Box::new(ConcreteType::TypeVar(
                        "T".to_string(),
                    ))))
                }
            }
            Expr::TupleLiteral(items) => {
                let tys = items
                    .iter()
                    .map(|i| self.infer_expr(i, env))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ConcreteType::Tuple(tys))
            }
            Expr::Lambda {
                params,
                return_type,
                body,
            } => {
                let mut lambda_env = env.enter_scope();
                let mut param_tys = Vec::new();
                for p in params {
                    let ty = if let Some(ref t) = p.type_annotation {
                        self.resolve_ast_type(t)?
                    } else {
                        ConcreteType::TypeVar(p.name.clone())
                    };
                    lambda_env.insert_var(p.name.clone(), ty.clone());
                    param_tys.push(ty);
                }

                let body_ty = self.infer_expr(body, &mut lambda_env)?;
                if let Some(ret) = return_type {
                    let ret_ty = self.resolve_ast_type(ret)?;
                    self.unify(&ret_ty, &body_ty, "Lambda return type")?;
                }

                Ok(ConcreteType::Function {
                    params: param_tys,
                    return_type: Box::new(body_ty),
                })
            }
            Expr::Match { subject, arms } => {
                let subj_ty = self.infer_expr(subject, env)?;
                if arms.is_empty() {
                    return Err("Match expression must have at least one arm".to_string());
                }

                let mut result_ty = None;
                for arm in arms {
                    let mut arm_env = env.enter_scope();
                    self.bind_pattern_with_type(&arm.pattern, &subj_ty, &mut arm_env)?;
                    let arm_body_ty = self.infer_expr(&arm.body, &mut arm_env)?;

                    if let Some(ref expected) = result_ty {
                        self.unify(expected, &arm_body_ty, "Match arm return value consistency")?;
                    } else {
                        result_ty = Some(arm_body_ty);
                    }
                }

                Ok(result_ty.unwrap_or(ConcreteType::Unit))
            }
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let _cond_ty = self.infer_expr(condition, env)?;

                let then_ty = self.infer_expr(then_branch, env)?;
                if let Some(else_expr) = else_branch {
                    let else_ty = self.infer_expr(else_expr, env)?;
                    if then_ty != ConcreteType::Any && else_ty != ConcreteType::Any {
                        let _ = self.unify(&then_ty, &else_ty, "If-else branch consistency");
                    }
                    Ok(then_ty)
                } else {
                    Ok(ConcreteType::Unit)
                }
            }
            Expr::Block(stmts) => {
                let mut block_env = env.enter_scope();
                let mut last_ty = ConcreteType::Unit;

                for stmt in stmts {
                    match stmt {
                        Statement::Let {
                            name,
                            type_annotation,
                            value,
                            ..
                        } => {
                            let val_ty = self.infer_expr(value, &mut block_env)?;
                            if let Some(annot) = type_annotation {
                                let annot_ty = self.resolve_ast_type(annot)?;
                                self.unify(&annot_ty, &val_ty, &format!("Let binding '{}'", name))?;
                                block_env.insert_var(name.clone(), annot_ty);
                            } else {
                                block_env.insert_var(name.clone(), val_ty);
                            }
                        }
                        Statement::LetPattern {
                            pattern,
                            type_annotation,
                            value,
                        } => {
                            let val_ty = self.infer_expr(value, &mut block_env)?;
                            if let Some(annot) = type_annotation {
                                let annot_ty = self.resolve_ast_type(annot)?;
                                self.unify(&annot_ty, &val_ty, "Let pattern binding")?;
                                self.bind_pattern_with_type(pattern, &annot_ty, &mut block_env)?;
                            } else {
                                self.bind_pattern_with_type(pattern, &val_ty, &mut block_env)?;
                            }
                        }
                        Statement::Assign { target, value } => {
                            let target_ty = self.infer_expr(target, &mut block_env)?;
                            let val_ty = self.infer_expr(value, &mut block_env)?;
                            self.unify(&target_ty, &val_ty, "Variable reassignment")?;
                            last_ty = ConcreteType::Unit;
                        }
                        Statement::Defer(expr) | Statement::ErrDefer(expr) => {
                            self.infer_expr(expr, &mut block_env)?;
                            last_ty = ConcreteType::Unit;
                        }
                        Statement::Expr(e) => {
                            last_ty = self.infer_expr(e, &mut block_env)?;
                        }
                        Statement::Return(Some(e)) => {
                            last_ty = self.infer_expr(e, &mut block_env)?;
                        }
                        Statement::Return(None) => {
                            last_ty = ConcreteType::Unit;
                        }
                    }
                }

                Ok(last_ty)
            }
            Expr::Async(body) => {
                let inner = self.infer_expr(body, env)?;
                match inner {
                    ConcreteType::Function {
                        params,
                        return_type,
                    } => Ok(ConcreteType::Function {
                        params,
                        return_type: Box::new(ConcreteType::Task(
                            return_type,
                            Box::new(ConcreteType::TypeVar("Error".to_string())),
                        )),
                    }),
                    _ => Ok(ConcreteType::Task(
                        Box::new(inner),
                        Box::new(ConcreteType::TypeVar("Error".to_string())),
                    )),
                }
            }
            Expr::Await(task_expr) => {
                let task_ty = self.infer_expr(task_expr, env)?;
                match task_ty {
                    ConcreteType::Task(ok, _) => Ok(*ok),
                    ConcreteType::Any => Ok(ConcreteType::Any),
                    other => Err(format!("Cannot await non-task type {}", other)),
                }
            }
            Expr::ConstructorCall { name, args } => {
                let arg_tys = args
                    .iter()
                    .map(|a| self.infer_expr(a, env))
                    .collect::<Result<Vec<_>, _>>()?;
                if let Some(sum_ty) = self.find_variant_sum_type(name) {
                    if let ConcreteType::Generic(type_params, inner) = sum_ty {
                        if let ConcreteType::SumType {
                            name,
                            variants,
                            type_args,
                        } = inner.as_ref()
                        {
                            let mut t_args = type_args.clone();
                            if t_args.is_empty() {
                                t_args =
                                    type_params.into_iter().map(ConcreteType::TypeVar).collect();
                            }
                            Ok(ConcreteType::SumType {
                                name: name.clone(),
                                variants: variants.clone(),
                                type_args: t_args,
                            })
                        } else {
                            Ok(inner.as_ref().clone())
                        }
                    } else {
                        Ok(sum_ty)
                    }
                } else if name == "Some" {
                    Ok(ConcreteType::Option(Box::new(
                        arg_tys.first().cloned().unwrap_or(ConcreteType::Unit),
                    )))
                } else if name == "Ok" {
                    Ok(ConcreteType::Result(
                        Box::new(arg_tys.first().cloned().unwrap_or(ConcreteType::Unit)),
                        Box::new(ConcreteType::TypeVar("E".to_string())),
                    ))
                } else if name == "Err" {
                    Ok(ConcreteType::Result(
                        Box::new(ConcreteType::TypeVar("T".to_string())),
                        Box::new(arg_tys.first().cloned().unwrap_or(ConcreteType::Unit)),
                    ))
                } else if let Some(ty) = env.lookup_type(name) {
                    if let ConcreteType::Generic(type_params, inner) = ty {
                        if let ConcreteType::SumType {
                            name,
                            variants,
                            mut type_args,
                        } = *inner
                        {
                            if type_args.is_empty() {
                                type_args =
                                    type_params.into_iter().map(ConcreteType::TypeVar).collect();
                            }
                            Ok(ConcreteType::SumType {
                                name,
                                variants,
                                type_args,
                            })
                        } else {
                            Ok(*inner)
                        }
                    } else {
                        Ok(ty)
                    }
                } else if let Some(fn_ty) = env.lookup_var(name) {
                    match fn_ty {
                        ConcreteType::Function {
                            params,
                            return_type,
                        } => {
                            for (arg, param) in args.iter().zip(params.iter()) {
                                let arg_ty = self.infer_expr(arg, env)?;
                                self.unify(param, &arg_ty, "Function argument")?;
                            }
                            Ok(*return_type)
                        }
                        ConcreteType::Generic(_, ref inner) => {
                            if let ConcreteType::Function {
                                params,
                                return_type,
                            } = inner.as_ref()
                            {
                                for (arg, param) in args.iter().zip(params.iter()) {
                                    let arg_ty = self.infer_expr(arg, env)?;
                                    self.unify(param, &arg_ty, "Function argument")?;
                                }
                                Ok((**return_type).clone())
                            } else {
                                Ok(fn_ty.clone())
                            }
                        }
                        _ => Ok(fn_ty.clone()),
                    }
                } else {
                    Ok(ConcreteType::SumType {
                        name: name.clone(),
                        type_args: vec![],
                        variants: HashMap::new(),
                    })
                }
            }
            Expr::Placeholder => Ok(ConcreteType::Any),
            Expr::Spawn(expr) => {
                let _ = self.infer_expr(expr, env)?;
                Ok(ConcreteType::Unit)
            }
            Expr::ChanSend { channel, value } => {
                let chan_ty = self.infer_expr(channel, env)?;
                let val_ty = self.infer_expr(value, env)?;
                match chan_ty {
                    ConcreteType::Channel(ref elem_ty) | ConcreteType::SendChannel(ref elem_ty) => {
                        self.unify(elem_ty, &val_ty, "Channel send value type")?;
                        self.validate_sendable_type(&val_ty)?;
                        Ok(ConcreteType::Bool)
                    }
                    ConcreteType::RecvChannel(_) => {
                        Err("Cannot send to a receive-only channel (RecvChannel)".to_string())
                    }
                    ConcreteType::Any => Ok(ConcreteType::Bool),
                    other => Err(format!("Cannot send to non-channel type {}", other)),
                }
            }
            Expr::ChanRecv(channel) => {
                let chan_ty = self.infer_expr(channel, env)?;
                match chan_ty {
                    ConcreteType::Channel(elem_ty) | ConcreteType::RecvChannel(elem_ty) => {
                        Ok(ConcreteType::Option(elem_ty))
                    }
                    ConcreteType::SendChannel(_) => {
                        Err("Cannot receive from a send-only channel (SendChannel)".to_string())
                    }
                    ConcreteType::Any => Ok(ConcreteType::Option(Box::new(ConcreteType::Any))),
                    other => Err(format!("Cannot receive from non-channel type {}", other)),
                }
            }
            Expr::Select { arms, default } => {
                let mut result_ty = None;
                for arm in arms {
                    let mut arm_env = env.enter_scope();
                    match &arm.kind {
                        SelectArmKind::Recv { binding, channel } => {
                            let ch_ty = self.infer_expr(channel, &mut arm_env)?;
                            let elem_ty = match ch_ty {
                                ConcreteType::Channel(inner) | ConcreteType::RecvChannel(inner) => ConcreteType::Option(inner),
                                ConcreteType::SendChannel(_) => return Err("Cannot receive from a send-only channel (SendChannel) in select".to_string()),
                                _ => ConcreteType::Option(Box::new(ConcreteType::Any)),
                            };
                            if let Some(var_name) = binding {
                                arm_env.insert_var(var_name.clone(), elem_ty);
                            }
                        }
                        SelectArmKind::Send { channel, value } => {
                            let ch_ty = self.infer_expr(channel, &mut arm_env)?;
                            let val_ty = self.infer_expr(value, &mut arm_env)?;
                            match &ch_ty {
                                ConcreteType::Channel(inner) | ConcreteType::SendChannel(inner) => {
                                    self.unify(inner, &val_ty, "Select send channel type")?;
                                    self.validate_sendable_type(&val_ty)?;
                                }
                                ConcreteType::RecvChannel(_) => return Err(
                                    "Cannot send to a receive-only channel (RecvChannel) in select"
                                        .to_string(),
                                ),
                                _ => {}
                            }
                        }
                        SelectArmKind::Timeout(expr) => {
                            let _ = self.infer_expr(expr, &mut arm_env)?;
                        }
                    }
                    let body_ty = self.infer_expr(&arm.body, &mut arm_env)?;
                    if let Some(ref expected) = result_ty {
                        self.unify(expected, &body_ty, "Select arm return type consistency")?;
                    } else {
                        result_ty = Some(body_ty);
                    }
                }
                if let Some(def) = default {
                    let def_ty = self.infer_expr(def, env)?;
                    if let Some(ref expected) = result_ty {
                        self.unify(expected, &def_ty, "Select default branch type consistency")?;
                    } else {
                        result_ty = Some(def_ty);
                    }
                }
                Ok(result_ty.unwrap_or(ConcreteType::Unit))
            }
            Expr::While {
                condition, body, ..
            } => {
                let cond_ty = self.infer_expr(condition, env)?;
                self.unify(&ConcreteType::Bool, &cond_ty, "While loop condition")?;
                let _ = self.infer_expr(body, env)?;
                Ok(ConcreteType::Unit)
            }
            Expr::ForIn {
                index_name,
                var_name,
                iterable,
                body,
                ..
            } => {
                let iter_ty = self.infer_expr(iterable, env)?;
                let mut loop_env = env.enter_scope();
                if let Some(idx) = index_name {
                    loop_env.insert_var(idx.clone(), ConcreteType::Int);
                }
                if let Some(elem) = self.is_iterator_type(&iter_ty) {
                    loop_env.insert_var(var_name.clone(), elem);
                } else {
                    match iter_ty {
                        ConcreteType::List(elem) | ConcreteType::Set(elem) => {
                            loop_env.insert_var(var_name.clone(), *elem);
                        }
                        ConcreteType::Channel(elem) | ConcreteType::RecvChannel(elem) => {
                            loop_env.insert_var(var_name.clone(), *elem);
                        }
                        ConcreteType::SendChannel(_) => {
                            return Err(
                                "Cannot iterate (for-in) over a send-only channel (SendChannel)"
                                    .to_string(),
                            );
                        }
                        ConcreteType::Map(k, _) => {
                            loop_env.insert_var(var_name.clone(), *k);
                        }
                        ConcreteType::String => {
                            loop_env.insert_var(var_name.clone(), ConcreteType::String);
                        }
                        _ => {
                            loop_env.insert_var(var_name.clone(), ConcreteType::Any);
                        }
                    }
                }
                let _ = self.infer_expr(body, &mut loop_env)?;
                Ok(ConcreteType::Unit)
            }
            Expr::Break(_) | Expr::Continue(_) => Ok(ConcreteType::Unit),
            Expr::AddressOf(inner) => {
                let ty = self.infer_expr(inner, env)?;
                Ok(ConcreteType::Pointer(Box::new(ty)))
            }
            Expr::Deref(inner) => {
                let ty = self.infer_expr(inner, env)?;
                match ty {
                    ConcreteType::Pointer(inner_ty) => Ok(*inner_ty),
                    ConcreteType::Any => Ok(ConcreteType::Any),
                    other => Err(format!("Cannot dereference non-pointer type '{}'", other)),
                }
            }
            Expr::Panic(inner) => {
                let _ = self.infer_expr(inner, env)?;
                Ok(ConcreteType::Unit)
            }
            Expr::Recover => Ok(ConcreteType::Option(Box::new(ConcreteType::Any))),
            Expr::Embed { is_binary, .. } => {
                if *is_binary {
                    Ok(ConcreteType::List(Box::new(ConcreteType::Int)))
                } else {
                    Ok(ConcreteType::String)
                }
            }
            _ => Ok(ConcreteType::Any),
        }
    }

    fn bind_pattern_with_type(
        &self,
        pattern: &Pattern,
        subject_ty: &ConcreteType,
        env: &mut TypeEnv,
    ) -> Result<(), String> {
        match pattern {
            Pattern::Wildcard => Ok(()),
            Pattern::Variable(name) => {
                env.insert_var(name.clone(), subject_ty.clone());
                Ok(())
            }
            Pattern::Literal(_) => Ok(()),
            Pattern::Constructor { name, patterns } => {
                if name == "Some" {
                    if let ConcreteType::Option(inner) = subject_ty {
                        if let Some(first_pat) = patterns.first() {
                            self.bind_pattern_with_type(first_pat, inner, env)?;
                        }
                        return Ok(());
                    } else if *subject_ty == ConcreteType::Any {
                        if let Some(first_pat) = patterns.first() {
                            self.bind_pattern_with_type(first_pat, &ConcreteType::Any, env)?;
                        }
                        return Ok(());
                    }
                }
                if name == "Ok" {
                    if let ConcreteType::Result(ok, _) = subject_ty {
                        if let Some(first_pat) = patterns.first() {
                            self.bind_pattern_with_type(first_pat, ok, env)?;
                        }
                        return Ok(());
                    } else if *subject_ty == ConcreteType::Any {
                        if let Some(first_pat) = patterns.first() {
                            self.bind_pattern_with_type(first_pat, &ConcreteType::Any, env)?;
                        }
                        return Ok(());
                    }
                }
                if name == "Err" {
                    if let ConcreteType::Result(_, err) = subject_ty {
                        if let Some(first_pat) = patterns.first() {
                            self.bind_pattern_with_type(first_pat, err, env)?;
                        }
                        return Ok(());
                    } else if *subject_ty == ConcreteType::Any {
                        if let Some(first_pat) = patterns.first() {
                            self.bind_pattern_with_type(first_pat, &ConcreteType::Any, env)?;
                        }
                        return Ok(());
                    }
                }

                let variants = match subject_ty {
                    ConcreteType::SumType { variants, .. } => Some(variants),
                    ConcreteType::Generic(_, inner) => {
                        if let ConcreteType::SumType { variants, .. } = &**inner {
                            Some(variants)
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                if let Some(variants_map) = variants
                    && let Some(shape) = variants_map.get(name)
                    && let VariantFieldTypes::Tuple(tys) = shape
                {
                    for (p, ty) in patterns.iter().zip(tys.iter()) {
                        self.bind_pattern_with_type(p, ty, env)?;
                    }
                }
                Ok(())
            }
            Pattern::Record {
                type_name, fields, ..
            } => {
                let rec_fields: Option<HashMap<String, ConcreteType>> = match subject_ty {
                    ConcreteType::Record(f) => Some(f.clone()),
                    ConcreteType::SumType { variants, .. } => {
                        let mut found = None;
                        for (vname, shape) in variants {
                            if let Some(tname) = type_name
                                && vname != tname
                            {
                                continue;
                            }
                            if let VariantFieldTypes::Record(f) = shape {
                                found = Some(f.clone());
                                break;
                            }
                        }
                        found
                    }
                    _ => {
                        if let Some(tname) = type_name {
                            if let Some(ConcreteType::SumType { variants, .. }) =
                                self.env.lookup_type(tname)
                            {
                                let mut found = None;
                                for (_, shape) in variants {
                                    if let VariantFieldTypes::Record(f) = shape {
                                        found = Some(f);
                                        break;
                                    }
                                }
                                found
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                };

                for (fname, pat) in fields {
                    let field_ty = rec_fields
                        .as_ref()
                        .and_then(|f| f.get(fname))
                        .cloned()
                        .unwrap_or(ConcreteType::Any);
                    if let Pattern::Variable(var_name) = pat {
                        env.insert_var(var_name.clone(), field_ty);
                    } else {
                        self.bind_pattern_with_type(pat, &field_ty, env)?;
                    }
                }
                Ok(())
            }
            Pattern::List { items, rest } => {
                let elem_ty = match subject_ty {
                    ConcreteType::List(elem) => *elem.clone(),
                    _ => ConcreteType::Any,
                };
                for item in items {
                    self.bind_pattern_with_type(item, &elem_ty, env)?;
                }
                if let Some(r) = rest {
                    self.bind_pattern_with_type(r, subject_ty, env)?;
                }
                Ok(())
            }
            Pattern::Tuple(items) => {
                let elem_tys = match subject_ty {
                    ConcreteType::Tuple(tys) => tys.clone(),
                    _ => vec![ConcreteType::Any; items.len()],
                };
                for (item, ty) in items.iter().zip(elem_tys.iter()) {
                    self.bind_pattern_with_type(item, ty, env)?;
                }
                Ok(())
            }
        }
    }

    /// Validates compile-time Sendable properties for CSP channel messages and concurrency boundaries.
    /// In accordance with Aura's Fearless Concurrency model, raw pointers (*T) are !Sendable
    /// because they permit unsynchronized shared mutable access across fibers.
    pub fn validate_sendable_type(&self, ty: &ConcreteType) -> Result<(), String> {
        match ty {
            ConcreteType::Pointer(inner) => Err(format!(
                "Concurrency Safety Error [E0401]: Cannot send raw pointer '*{}' across CSP channel. Raw pointers violate thread isolation. Use safe value types or synchronized channels.",
                inner
            )),
            ConcreteType::Option(inner) => self.validate_sendable_type(inner),
            ConcreteType::Result(ok, err) => {
                self.validate_sendable_type(ok)?;
                self.validate_sendable_type(err)
            }
            ConcreteType::Task(ok, err) => {
                self.validate_sendable_type(ok)?;
                self.validate_sendable_type(err)
            }
            ConcreteType::List(elem) => self.validate_sendable_type(elem),
            ConcreteType::Tuple(elems) => {
                for e in elems {
                    self.validate_sendable_type(e)?;
                }
                Ok(())
            }
            ConcreteType::Map(k, v) => {
                self.validate_sendable_type(k)?;
                self.validate_sendable_type(v)
            }
            ConcreteType::Record(fields) => {
                for f_ty in fields.values() {
                    self.validate_sendable_type(f_ty)?;
                }
                Ok(())
            }
            ConcreteType::SumType { variants, .. } => {
                for v_fields in variants.values() {
                    match v_fields {
                        VariantFieldTypes::Unit => {}
                        VariantFieldTypes::Tuple(tys) => {
                            for ty in tys {
                                self.validate_sendable_type(ty)?;
                            }
                        }
                        VariantFieldTypes::Record(fields) => {
                            for ty in fields.values() {
                                self.validate_sendable_type(ty)?;
                            }
                        }
                    }
                }
                Ok(())
            }
            ConcreteType::Channel(elem)
            | ConcreteType::SendChannel(elem)
            | ConcreteType::RecvChannel(elem) => self.validate_sendable_type(elem),
            _ => Ok(()),
        }
    }

    pub fn unify(
        &self,
        expected: &ConcreteType,
        actual: &ConcreteType,
        context: &str,
    ) -> Result<(), String> {
        if expected == actual
            || matches!(expected, ConcreteType::Any)
            || matches!(actual, ConcreteType::Any)
        {
            return Ok(());
        }

        match (expected, actual) {
            (ConcreteType::TypeVar(_), _) | (_, ConcreteType::TypeVar(_)) => Ok(()),
            (ConcreteType::Option(e), ConcreteType::Option(a)) => self.unify(e, a, context),
            (ConcreteType::Result(ok1, err1), ConcreteType::Result(ok2, err2)) => {
                self.unify(ok1, ok2, context)?;
                self.unify(err1, err2, context)
            }
            (ConcreteType::Task(ok1, err1), ConcreteType::Task(ok2, err2)) => {
                self.unify(ok1, ok2, context)?;
                self.unify(err1, err2, context)
            }
            (ConcreteType::Task(ok, _), actual) => self.unify(ok, actual, context),
            (ConcreteType::Pointer(e), ConcreteType::Pointer(a)) => self.unify(e, a, context),
            (
                ConcreteType::Interface {
                    name: exp_name,
                    methods: exp_methods,
                    ..
                },
                ConcreteType::Record(act_fields),
            ) => {
                let is_iterator_iface = exp_name == "Iterator" || exp_name == "Interator";
                for (mname, (mparams, mret)) in exp_methods {
                    if is_iterator_iface && mname != "next" {
                        continue;
                    }
                    let exp_fn = ConcreteType::Function {
                        params: mparams.clone(),
                        return_type: Box::new(mret.clone()),
                    };
                    if let Some(act_field) = act_fields.get(mname) {
                        self.unify(
                            &exp_fn,
                            act_field,
                            &format!("{} interface method '{}'", context, mname),
                        )?;
                    } else if let Some(method_fn) = self.lookup_method_for_type(actual, mname) {
                        self.unify(
                            &exp_fn,
                            &method_fn,
                            &format!("{} interface method '{}'", context, mname),
                        )?;
                    } else {
                        return Err(format!(
                            "Type is missing required interface method '{}' in {}",
                            mname, context
                        ));
                    }
                }
                Ok(())
            }
            (
                ConcreteType::Interface {
                    name: exp_name,
                    methods: exp_methods,
                    ..
                },
                ConcreteType::Pointer(inner),
            ) => {
                let is_iterator_iface = exp_name == "Iterator" || exp_name == "Interator";
                for (mname, (mparams, mret)) in exp_methods {
                    if is_iterator_iface && mname != "next" {
                        continue;
                    }
                    let exp_fn = ConcreteType::Function {
                        params: mparams.clone(),
                        return_type: Box::new(mret.clone()),
                    };
                    if let Some(method_fn) = self.lookup_method_for_type(actual, mname) {
                        self.unify(
                            &exp_fn,
                            &method_fn,
                            &format!("{} interface method '{}'", context, mname),
                        )?;
                    } else if let Some(method_fn) = self.lookup_method_for_type(inner, mname) {
                        self.unify(
                            &exp_fn,
                            &method_fn,
                            &format!("{} interface method '{}'", context, mname),
                        )?;
                    } else if let ConcreteType::Record(act_fields) = &**inner {
                        if let Some(act_field) = act_fields.get(mname) {
                            self.unify(
                                &exp_fn,
                                act_field,
                                &format!("{} interface method '{}'", context, mname),
                            )?;
                        } else {
                            return Err(format!(
                                "Pointer type is missing required interface method '{}' in {}",
                                mname, context
                            ));
                        }
                    } else {
                        return Err(format!(
                            "Pointer type is missing required interface method '{}' in {}",
                            mname, context
                        ));
                    }
                }
                Ok(())
            }
            (
                ConcreteType::Interface {
                    name: exp_name,
                    methods: exp_methods,
                    ..
                },
                ConcreteType::Interface {
                    methods: act_methods,
                    ..
                },
            ) => {
                let is_iterator_iface = exp_name == "Iterator" || exp_name == "Interator";
                for (mname, (mparams, mret)) in exp_methods {
                    if is_iterator_iface && mname != "next" {
                        continue;
                    }
                    if let Some((act_params, act_ret)) = act_methods.get(mname) {
                        let exp_fn = ConcreteType::Function {
                            params: mparams.clone(),
                            return_type: Box::new(mret.clone()),
                        };
                        let act_fn = ConcreteType::Function {
                            params: act_params.clone(),
                            return_type: Box::new(act_ret.clone()),
                        };
                        self.unify(
                            &exp_fn,
                            &act_fn,
                            &format!("{} interface method '{}'", context, mname),
                        )?;
                    } else {
                        return Err(format!(
                            "Interface is missing required method '{}' in {}",
                            mname, context
                        ));
                    }
                }
                Ok(())
            }
            (
                ConcreteType::Interface {
                    name: exp_name,
                    methods: exp_methods,
                    ..
                },
                ConcreteType::List(elem),
            ) => {
                if exp_name == "Iterator" || exp_name == "Interator" {
                    if let Some((_, ret)) = exp_methods.get("next")
                        && let ConcreteType::Option(inner) = ret
                    {
                        return self.unify(inner, elem, context);
                    }
                    Ok(())
                } else {
                    Err(format!(
                        "Type mismatch in {}: expected interface '{}', found List",
                        context, exp_name
                    ))
                }
            }
            (ConcreteType::Channel(e), ConcreteType::Channel(a)) => self.unify(e, a, context),
            (ConcreteType::SendChannel(e), ConcreteType::SendChannel(a)) => {
                self.unify(e, a, context)
            }
            (ConcreteType::RecvChannel(e), ConcreteType::RecvChannel(a)) => {
                self.unify(e, a, context)
            }
            (ConcreteType::SendChannel(e), ConcreteType::Channel(a)) => self.unify(e, a, context),
            (ConcreteType::RecvChannel(e), ConcreteType::Channel(a)) => self.unify(e, a, context),
            (ConcreteType::Channel(e), ConcreteType::SendChannel(a)) => self.unify(e, a, context),
            (ConcreteType::Channel(e), ConcreteType::RecvChannel(a)) => self.unify(e, a, context),
            (ConcreteType::List(e), ConcreteType::List(a)) => self.unify(e, a, context),
            (ConcreteType::Map(k1, v1), ConcreteType::Map(k2, v2)) => {
                self.unify(k1, k2, context)?;
                self.unify(v1, v2, context)
            }
            (ConcreteType::Set(e), ConcreteType::Set(a)) => self.unify(e, a, context),
            (ConcreteType::Tuple(t1), ConcreteType::Tuple(t2)) => {
                if t1.len() != t2.len() {
                    return Err(format!("Tuple length mismatch in {}", context));
                }
                for (e1, e2) in t1.iter().zip(t2.iter()) {
                    self.unify(e1, e2, context)?;
                }
                Ok(())
            }
            (
                ConcreteType::Function {
                    params: p1,
                    return_type: r1,
                },
                ConcreteType::Function {
                    params: p2,
                    return_type: r2,
                },
            ) => {
                if p1.len() != p2.len() {
                    return Err(format!("Function arity mismatch in {}", context));
                }
                for (a1, a2) in p1.iter().zip(p2.iter()) {
                    self.unify(a1, a2, context)?;
                }
                self.unify(r1, r2, context)
            }
            (ConcreteType::Generic(p1, i1), ConcreteType::Generic(p2, i2)) => {
                if p1.len() != p2.len() {
                    return Err(format!("Generic arity mismatch in {}", context));
                }
                self.unify(i1, i2, context)
            }
            (ConcreteType::Record(exp_fields), ConcreteType::Record(act_fields)) => {
                for (k, exp_val) in exp_fields {
                    if let Some(act_val) = act_fields.get(k) {
                        self.unify(exp_val, act_val, &format!("{} field '{}'", context, k))?;
                    } else {
                        return Err(format!("Missing field '{}' in record", k));
                    }
                }
                Ok(())
            }
            (
                ConcreteType::SumType {
                    name: n1,
                    type_args: a1,
                    ..
                },
                ConcreteType::SumType {
                    name: n2,
                    type_args: a2,
                    ..
                },
            ) => {
                if n1 == n2 && a1.len() == a2.len() {
                    for (t1, t2) in a1.iter().zip(a2.iter()) {
                        self.unify(t1, t2, context)?;
                    }
                    Ok(())
                } else {
                    Err(format!(
                        "Type mismatch in {}: expected sum type '{}', found '{}'",
                        context, n1, n2
                    ))
                }
            }
            _ => Err(format!(
                "Type mismatch in {}: expected '{}', found '{}'",
                context, expected, actual
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn check_source(src: &str) -> Result<(), String> {
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let module = parser.parse_module()?;
        let mut checker = TypeChecker::new();
        checker.check_module(&module)
    }

    #[test]
    fn test_type_env_scopes() {
        let mut env = TypeEnv::new();
        env.insert_var("x".to_string(), ConcreteType::Int);
        assert_eq!(env.lookup_var("x"), Some(ConcreteType::Int));

        let mut child = env.enter_scope();
        child.insert_var("y".to_string(), ConcreteType::String);
        child.insert_var("x".to_string(), ConcreteType::Float); // shadowing

        assert_eq!(child.lookup_var("y"), Some(ConcreteType::String));
        assert_eq!(child.lookup_var("x"), Some(ConcreteType::Float));

        assert_eq!(env.lookup_var("x"), Some(ConcreteType::Int));
        assert_eq!(env.lookup_var("y"), None);
    }

    #[test]
    fn test_unification() {
        let checker = TypeChecker::new();

        // Exact matches
        assert!(
            checker
                .unify(&ConcreteType::Int, &ConcreteType::Int, "test")
                .is_ok()
        );
        assert!(
            checker
                .unify(&ConcreteType::String, &ConcreteType::String, "test")
                .is_ok()
        );

        // Any and TypeVar
        assert!(
            checker
                .unify(&ConcreteType::Any, &ConcreteType::Int, "test")
                .is_ok()
        );
        assert!(
            checker
                .unify(&ConcreteType::Int, &ConcreteType::Any, "test")
                .is_ok()
        );
        assert!(
            checker
                .unify(
                    &ConcreteType::TypeVar("T".to_string()),
                    &ConcreteType::Int,
                    "test"
                )
                .is_ok()
        );

        // List & Option
        assert!(
            checker
                .unify(
                    &ConcreteType::List(Box::new(ConcreteType::Int)),
                    &ConcreteType::List(Box::new(ConcreteType::Int)),
                    "test"
                )
                .is_ok()
        );

        // Records
        let mut r1 = HashMap::new();
        r1.insert("a".to_string(), ConcreteType::Int);
        let mut r2 = HashMap::new();
        r2.insert("a".to_string(), ConcreteType::Int);
        r2.insert("b".to_string(), ConcreteType::String);

        assert!(
            checker
                .unify(&ConcreteType::Record(r1), &ConcreteType::Record(r2), "test")
                .is_ok()
        );

        // Mismatches
        assert!(
            checker
                .unify(&ConcreteType::Int, &ConcreteType::String, "test")
                .is_err()
        );
    }

    #[test]
    fn test_typecheck_functions_and_variables() {
        let src = r#"
            fn add(a: Int, b: Int): Int => a + b;

            fn test_main(): Int {
                let x = 10;
                let y = 20;
                return add(x, y);
            }
        "#;
        assert!(check_source(src).is_ok());
    }

    #[test]
    fn test_typecheck_records_and_destructuring() {
        let src = r#"
            type User = {
                id: Int,
                name: String
            }

            fn get_name(u: User): String => u.name;

            fn main(): String {
                let u = { id: 1, name: "Alice" };
                return get_name(u);
            }
        "#;
        assert!(check_source(src).is_ok());
    }

    #[test]
    fn test_typecheck_sum_types_and_pattern_matching() {
        let src = r#"
            type Shape =
                | Circle(Float)
                | Rect { w: Float, h: Float }
                | Point;

            fn area(s: Shape): Float =>
                match s {
                    Circle(r) => 3.14 * r * r,
                    Rect { w, h } => w * h,
                    Point => 0.0
                }
        "#;
        assert!(check_source(src).is_ok());
    }

    #[test]
    fn test_typecheck_pipelines() {
        let src = r#"
            fn inc(x: Int): Int => x + 1;
            fn double(x: Int): Int => x * 2;

            fn calc(n: Int): Int =>
                n
                |> inc
                |> double;
        "#;
        assert!(check_source(src).is_ok());
    }

    #[test]
    fn test_typecheck_csp_concurrency() {
        let src = r#"
            fn worker(ch: Channel<Int>) {
                ch <- 42;
            }

            fn main() {
                let ch = Channel.make();
                spawn worker(ch);
                let wg = WaitGroup.new();
                wg.add(1);
                wg.done();
            }
        "#;
        assert!(check_source(src).is_ok());
    }

    #[test]
    fn test_typecheck_errors() {
        let bad_type = r#"
            fn add(a: Int, b: Int): Int => "string";
        "#;
        assert!(check_source(bad_type).is_err());

        let undefined_var = r#"
            fn test(): Int => unknown_var;
        "#;
        assert!(check_source(undefined_var).is_err());
    }
}
