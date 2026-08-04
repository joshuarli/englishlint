use englishlint::config::Config;
use englishlint::lint_directory;
use std::env;
use std::path::PathBuf;
use std::process;

fn main() {
    let args = match englishlint::cli::parse(env::args().skip(1)) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("englishlint: {error}\nusage: englishlint [OPTIONS] <directory>");
            process::exit(2);
        }
    };
    if args.help {
        println!("{}", englishlint::cli::help());
        return;
    }
    if args.version {
        println!("englishlint {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let Some(root_arg) = args.root else {
        eprintln!("usage: englishlint [OPTIONS] <directory>");
        process::exit(2);
    };
    let root = match root_arg.canonicalize() {
        Ok(path) => path,
        Err(error) => {
            if error.kind() == std::io::ErrorKind::NotFound {
                eprintln!("englishlint: '{}' is not a directory", root_arg.display());
            } else {
                eprintln!(
                    "englishlint: cannot access '{}': {error}",
                    root_arg.display()
                );
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
            print!(
                "{}",
                englishlint::output::render(&file.source, &root, &diagnostic)
            );
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
