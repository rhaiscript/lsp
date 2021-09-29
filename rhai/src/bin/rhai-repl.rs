use rhai::{Dynamic, Engine, EvalAltResult, Module, Scope, AST};

use std::{
    env,
    fs::File,
    io::{stdin, stdout, Read, Write},
    path::Path,
    process::exit,
};

/// Pretty-print error.
fn print_error(input: &str, mut err: EvalAltResult) {
    let lines: Vec<_> = input.trim().split('\n').collect();
    let pos = err.take_position();

    let line_no = if lines.len() > 1 {
        if pos.is_none() {
            "".to_string()
        } else {
            format!("{}: ", pos.line().unwrap())
        }
    } else {
        "".to_string()
    };

    // Print error position
    if pos.is_none() {
        // No position
        println!("{}", err);
    } else {
        // Specific position - print line text
        println!("{}{}", line_no, lines[pos.line().unwrap() - 1]);

        // Display position marker
        println!(
            "{0:>1$} {2}",
            "^",
            line_no.len() + pos.position().unwrap(),
            err
        );
    }
}

/// Print help text.
fn print_help() {
    println!("help       => print this help");
    println!("quit, exit => quit");
    println!("scope      => print all variables in the scope");
    #[cfg(feature = "metadata")]
    println!("functions  => print all functions defined");
    #[cfg(feature = "metadata")]
    println!("json       => output all functions in JSON format");
    println!("ast        => print the last AST (optimized)");
    println!("astu       => print the last raw, un-optimized AST");
    println!(r"end a line with '\' to continue to the next line.");
    println!();
}

fn main() {
    let title = format!("Rhai REPL tool (version {})", env!("CARGO_PKG_VERSION"));
    println!("{}", title);
    println!("{0:=<1$}", "", title.len());
    print_help();

    // Initialize scripting engine
    let mut engine = Engine::new();

    #[cfg(not(feature = "no_module"))]
    #[cfg(not(feature = "no_std"))]
    {
        // Load init scripts
        let mut contents = String::new();
        let mut has_init_scripts = false;

        for filename in env::args().skip(1) {
            let filename = match Path::new(&filename).canonicalize() {
                Err(err) => {
                    eprintln!("Error script file path: {}\n{}", filename, err);
                    exit(1);
                }
                Ok(f) => {
                    match f.strip_prefix(std::env::current_dir().unwrap().canonicalize().unwrap()) {
                        Ok(f) => f.into(),
                        _ => f,
                    }
                }
            };

            contents.clear();

            let mut f = match File::open(&filename) {
                Err(err) => {
                    eprintln!(
                        "Error reading script file: {}\n{}",
                        filename.to_string_lossy(),
                        err
                    );
                    exit(1);
                }
                Ok(f) => f,
            };

            if let Err(err) = f.read_to_string(&mut contents) {
                println!(
                    "Error reading script file: {}\n{}",
                    filename.to_string_lossy(),
                    err
                );
                exit(1);
            }

            let module = match engine
                .compile(&contents)
                .map_err(|err| err.into())
                .and_then(|mut ast| {
                    ast.set_source(filename.to_string_lossy().to_string());
                    Module::eval_ast_as_new(Default::default(), &ast, &engine)
                }) {
                Err(err) => {
                    let filename = filename.to_string_lossy();

                    eprintln!("{:=<1$}", "", filename.len());
                    eprintln!("{}", filename);
                    eprintln!("{:=<1$}", "", filename.len());
                    eprintln!("");

                    print_error(&contents, *err);
                    exit(1);
                }
                Ok(m) => m,
            };

            engine.register_global_module(module.into());

            has_init_scripts = true;

            println!("Script '{}' loaded.", filename.to_string_lossy());
        }

        if has_init_scripts {
            println!();
        }
    }

    // Setup Engine
    #[cfg(not(feature = "no_optimize"))]
    engine.set_optimization_level(rhai::OptimizationLevel::None);

    // Set a file module resolver without caching
    #[cfg(not(feature = "no_module"))]
    #[cfg(not(feature = "no_std"))]
    {
        let mut resolver = rhai::module_resolvers::FileModuleResolver::new();
        resolver.enable_cache(false);
        engine.set_module_resolver(resolver);
    }

    // Make Engine immutable
    let engine = engine;

    // Create scope
    let mut scope = Scope::new();

    // REPL loop
    let mut input = String::new();
    let mut main_ast: AST = Default::default();
    let mut ast_u: AST = Default::default();
    let mut ast: AST = Default::default();

    'main_loop: loop {
        print!("rhai-repl> ");
        stdout().flush().expect("couldn't flush stdout");

        input.clear();

        loop {
            match stdin().read_line(&mut input) {
                Ok(0) => break 'main_loop,
                Ok(_) => (),
                Err(err) => panic!("input error: {}", err),
            }

            let line = input.as_str().trim_end();

            // Allow line continuation
            if line.ends_with('\\') {
                let len = line.len();
                input.truncate(len - 1);
                input.push('\n');
            } else {
                break;
            }

            print!("> ");
            stdout().flush().expect("couldn't flush stdout");
        }

        let script = input.trim();

        if script.is_empty() {
            continue;
        }

        // Implement standard commands
        match script {
            "help" => {
                print_help();
                continue;
            }
            "exit" | "quit" => break, // quit
            "scope" => {
                scope
                    .iter_raw()
                    .enumerate()
                    .for_each(|(i, (name, constant, value))| {
                        #[cfg(not(feature = "no_closure"))]
                        let value_is_shared = if value.is_shared() { " (shared)" } else { "" };
                        #[cfg(feature = "no_closure")]
                        let value_is_shared = "";

                        println!(
                            "[{}] {}{}{} = {:?}",
                            i + 1,
                            if constant { "const " } else { "" },
                            name,
                            value_is_shared,
                            *value.read_lock::<Dynamic>().unwrap(),
                        )
                    });
                println!();
                continue;
            }
            "astu" => {
                // print the last un-optimized AST
                println!("{:#?}\n", ast_u);
                continue;
            }
            "ast" => {
                // print the last AST
                println!("{:#?}\n", ast);
                continue;
            }
            #[cfg(feature = "metadata")]
            "functions" => {
                // print a list of all registered functions
                engine
                    .gen_fn_signatures(false)
                    .into_iter()
                    .for_each(|f| println!("{}", f));

                #[cfg(not(feature = "no_function"))]
                main_ast.iter_functions().for_each(|f| println!("{}", f));

                println!();
                continue;
            }
            #[cfg(feature = "metadata")]
            "json" => {
                println!(
                    "{}",
                    engine
                        .gen_fn_metadata_with_ast_to_json(&main_ast, true)
                        .unwrap()
                );
                continue;
            }
            _ => (),
        }

        match engine
            .compile_with_scope(&scope, &script)
            .map_err(Into::into)
            .and_then(|r| {
                ast_u = r.clone();

                #[cfg(not(feature = "no_optimize"))]
                {
                    ast = engine.optimize_ast(&scope, r, rhai::OptimizationLevel::Simple);
                }

                #[cfg(feature = "no_optimize")]
                {
                    ast = r;
                }

                // Merge the AST into the main
                main_ast += ast.clone();

                // Evaluate
                engine.eval_ast_with_scope::<Dynamic>(&mut scope, &main_ast)
            }) {
            Ok(result) if !result.is::<()>() => {
                println!("=> {:?}", result);
                println!();
            }
            Ok(_) => (),
            Err(err) => {
                println!();
                print_error(&input, *err);
                println!();
            }
        }

        // Throw away all the statements, leaving only the functions
        main_ast.clear_statements();
    }
}
