use englishlint::config::Config;
use englishlint::diagnostic::Severity;
use englishlint::lint_directory;
use englishlint::rules::RuleId;
use std::collections::BTreeMap;
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
    let mut rule_severities = BTreeMap::<RuleId, Severity>::new();
    for file in results {
        total += file.diagnostics.len();
        for diagnostic in &file.diagnostics {
            rule_severities
                .entry(diagnostic.rule)
                .and_modify(|severity| *severity = (*severity).min(diagnostic.severity))
                .or_insert(diagnostic.severity);
        }
        if !file.diagnostics.is_empty() {
            print!(
                "{}",
                englishlint::output::render_file(&file.source, &root, &file.diagnostics)
            );
        }
    }
    if total == 0 {
        println!("englishlint: no findings");
        process::exit(0);
    }
    let summary = rule_severities.into_iter().collect::<Vec<_>>();
    print!("{}", englishlint::output::render_rule_summary(&summary));
    eprintln!(
        "englishlint: {total} finding{}",
        if total == 1 { "" } else { "s" }
    );
    process::exit(1);
}
