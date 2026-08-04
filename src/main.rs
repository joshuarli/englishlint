use englishlint::config::Config;
use englishlint::diagnostic::Suggestion;
use englishlint::lint_directory;
use std::env;
use std::path::PathBuf;
use std::process;

struct Args {
    config: Option<PathBuf>,
    root: Option<PathBuf>,
    help: bool,
    version: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        config: None,
        root: None,
        help: false,
        version: false,
    };
    let values: Vec<String> = env::args().skip(1).collect();
    let mut index = 0;
    while index < values.len() {
        match values[index].as_str() {
            "-h" | "--help" => args.help = true,
            "--version" => args.version = true,
            "-c" | "--config" => {
                index += 1;
                let Some(path) = values.get(index) else {
                    return Err("--config requires a file".into());
                };
                args.config = Some(PathBuf::from(path));
            }
            value if value.starts_with('-') => return Err(format!("unknown option '{value}'")),
            value => {
                if args.root.is_some() {
                    return Err("expected one directory".into());
                }
                args.root = Some(PathBuf::from(value));
            }
        }
        index += 1;
    }
    Ok(args)
}

fn print_help() {
    println!("englishlint - deterministic technical English linter\n\nUsage:\n  englishlint [OPTIONS] <directory>\n\nOptions:\n  -c, --config FILE  Read FILE instead of ./englishlint.ini\n  -h, --help         Show this help\n      --version      Show the version\n\nExit codes:\n  0  no findings\n  1  findings\n  2  usage, configuration, or file error");
}

fn main() {
    let args = match parse_args() {
        Ok(args) => args,
        Err(error) => {
            eprintln!("englishlint: {error}\nusage: englishlint [OPTIONS] <directory>");
            process::exit(2);
        }
    };
    if args.help {
        print_help();
        return;
    }
    if args.version {
        println!("englishlint {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let Some(root) = args.root else {
        eprintln!("usage: englishlint [OPTIONS] <directory>");
        process::exit(2);
    };
    let root = match root.canonicalize() {
        Ok(path) => path,
        Err(error) => {
            if error.kind() == std::io::ErrorKind::NotFound {
                eprintln!("englishlint: '{}' is not a directory", root.display());
            } else {
                eprintln!("englishlint: cannot access '{}': {error}", root.display());
            }
            process::exit(2);
        }
    };
    if !root.is_dir() {
        eprintln!("englishlint: '{}' is not a directory", root.display());
        process::exit(2);
    }
    let config_path = args.config.unwrap_or_else(|| {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("englishlint.ini")
    });
    let config = match Config::read(&config_path) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("englishlint: {error}");
            process::exit(2);
        }
    };
    let results = match lint_directory(&root, &config) {
        Ok(results) => results,
        Err(error) => {
            eprintln!("englishlint: {error}");
            process::exit(2);
        }
    };
    let mut total = 0;
    for file in results {
        for diagnostic in file.diagnostics {
            total += 1;
            let location = diagnostic.location(&file.source);
            println!(
                "{}:{}:{}: {} {}",
                file.source.display_path(&root),
                location.line,
                location.column,
                diagnostic.rule,
                diagnostic.message
            );
            if let Some(Suggestion::Message(message)) = diagnostic.suggestion {
                println!("  suggestion: {message}");
            }
        }
    }
    if total == 0 {
        println!("englishlint: no findings");
        process::exit(0);
    }
    eprintln!(
        "englishlint: {total} finding{}",
        if total == 1 { "" } else { "s" }
    );
    process::exit(1);
}
