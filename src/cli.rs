use std::path::PathBuf;

#[derive(Debug, Eq, PartialEq)]
pub struct Args {
    pub config: Option<PathBuf>,
    pub root: Option<PathBuf>,
    pub help: bool,
    pub version: bool,
}

pub fn parse(values: impl IntoIterator<Item = String>) -> Result<Args, String> {
    let mut args = Args {
        config: None,
        root: None,
        help: false,
        version: false,
    };
    let values: Vec<String> = values.into_iter().collect();
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

pub fn help() -> &'static str {
    "englishlint - deterministic technical English linter\n\nUsage:\n  englishlint [OPTIONS] <directory>\n\nOptions:\n  -c, --config FILE  Read FILE instead of ./englishlint.ini\n  -h, --help         Show this help\n      --version      Show the version\n\nExit codes:\n  0  no findings\n  1  findings\n  2  usage, configuration, or file error"
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_options_and_rejects_unknown_flags() {
        let args = parse(["--config".into(), "x.ini".into(), "docs".into()]).unwrap();
        assert_eq!(args.config, Some(PathBuf::from("x.ini")));
        assert_eq!(args.root, Some(PathBuf::from("docs")));
        assert!(parse(["--nope".into()]).is_err());
    }
}
