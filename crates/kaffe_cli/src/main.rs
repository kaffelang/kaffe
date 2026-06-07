use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "parse" => cmd_parse(&args[2]),
        "build" => cmd_build(&args),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
            process::exit(1);
        }
    }
}

fn cmd_parse(path: &str) {
    let source = read_file(path);
    match kaffe_parser::parse(&source) {
        Ok(module) => {
            println!("{}", serde_json::to_string_pretty(&module).unwrap());
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    }
}

fn cmd_build(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Usage: kaffe build <file> [--out <output>]");
        process::exit(1);
    }

    let input = &args[2];
    let output = match args.get(3).map(String::as_str) {
        None => None,
        Some("--out") => match args.get(4) {
            Some(path) if args.len() == 5 => Some(path.as_str()),
            Some(_) => {
                eprintln!("Unexpected arguments after --out");
                print_usage();
                process::exit(1);
            }
            None => {
                eprintln!("Missing output path after --out");
                print_usage();
                process::exit(1);
            }
        },
        Some(_) => {
            eprintln!("Unexpected arguments");
            print_usage();
            process::exit(1);
        }
    };

    let source = read_file(input);
    match kaffe_parser::parse(&source) {
        Ok(module) => {
            let ts = kaffe_emit_ts::emit_module(&module);
            if let Some(path) = output {
                if let Some(parent) = std::path::Path::new(path).parent() {
                    if !parent.as_os_str().is_empty() {
                        fs::create_dir_all(parent).unwrap_or_else(|err| {
                            eprintln!("Failed to create output directory: {}", err);
                            process::exit(1);
                        });
                    }
                }
                fs::write(path, &ts).unwrap_or_else(|err| {
                    eprintln!("Failed to write {}: {}", path, err);
                    process::exit(1);
                });
                println!("Built: {}", path);
            } else {
                print!("{}", ts);
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    }
}

fn read_file(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| {
        eprintln!("Failed to read {}: {}", path, err);
        process::exit(1);
    })
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  kaffe parse <file.kaf>");
    eprintln!("  kaffe build <file.kaf> [--out <output.ts>]");
}
