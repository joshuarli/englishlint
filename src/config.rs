use crate::rules::RuleId;
use crate::text::TextMode;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct Config {
    pub default_mode: TextMode,
    pub descriptive_limit: usize,
    pub procedural_limit: usize,
    pub procedural_headings: Vec<String>,
    pub ignored_rules: HashSet<RuleId>,
    pub ignored_words: HashSet<String>,
    pub glossary: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_mode: TextMode::Descriptive,
            descriptive_limit: 25,
            procedural_limit: 20,
            procedural_headings: [
                "install",
                "setup",
                "configure",
                "configuration",
                "usage",
                "troubleshoot",
                "troubleshooting",
                "migration",
                "migrate",
                "deploy",
                "deployment",
                "procedure",
                "procedures",
                "steps",
                "getting started",
                "quickstart",
                "start",
                "upgrade",
                "backup",
                "reset",
                "run",
                "commands",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            ignored_rules: HashSet::new(),
            ignored_words: HashSet::new(),
            glossary: HashMap::new(),
        }
    }
}

impl Config {
    pub fn read(path: &Path) -> Result<Self, String> {
        let mut config = Self::default();
        if !path.exists() {
            return Ok(config);
        }
        let source = fs::read_to_string(path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let mut section = String::new();
        for (line_number, raw) in source.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = line[1..line.len() - 1].trim().to_ascii_lowercase();
                continue;
            }
            let Some(equal) = line.find('=') else {
                return Err(format!(
                    "{}:{}: expected section or key = value",
                    path.display(),
                    line_number + 1
                ));
            };
            let key = line[..equal].trim().to_ascii_lowercase();
            let value = line[equal + 1..].trim();
            match section.as_str() {
                "lint" => config.set_lint(&key, value, path, line_number + 1)?,
                "ignore" => config.set_ignore(&key, value, path, line_number + 1)?,
                "glossary" => {
                    if !matches!(key.as_str(), "check" | "config" | "delete") {
                        return Err(format!(
                            "{}:{}: unknown glossary concept '{}'",
                            path.display(),
                            line_number + 1,
                            key
                        ));
                    }
                    config.glossary.insert(key, unquote(value));
                }
                _ => {
                    return Err(format!(
                        "{}:{}: unknown section [{}]",
                        path.display(),
                        line_number + 1,
                        section
                    ))
                }
            }
        }
        Ok(config)
    }

    fn set_lint(&mut self, key: &str, value: &str, path: &Path, line: usize) -> Result<(), String> {
        match key {
            "default_type" => {
                self.default_mode = TextMode::parse(value).ok_or_else(|| {
                    format!(
                        "{}:{}: default_type must be procedural or descriptive",
                        path.display(),
                        line
                    )
                })?
            }
            "descriptive_limit" => {
                self.descriptive_limit = parse_positive(value, "descriptive_limit", path, line)?
            }
            "procedural_limit" => {
                self.procedural_limit = parse_positive(value, "procedural_limit", path, line)?
            }
            "procedural_headings" => self.procedural_headings = split_list(value),
            "ignore_rules" => self.add_rules(value, path, line)?,
            "ignore_words" => self.ignored_words.extend(split_list(value)),
            _ => {
                return Err(format!(
                    "{}:{}: unknown [lint] key '{}'",
                    path.display(),
                    line,
                    key
                ))
            }
        }
        Ok(())
    }

    fn set_ignore(
        &mut self,
        key: &str,
        value: &str,
        path: &Path,
        line: usize,
    ) -> Result<(), String> {
        match key {
            "rules" => self.add_rules(value, path, line)?,
            "words" => self.ignored_words.extend(split_list(value)),
            _ => {
                return Err(format!(
                    "{}:{}: unknown [ignore] key '{}'",
                    path.display(),
                    line,
                    key
                ))
            }
        }
        Ok(())
    }

    fn add_rules(&mut self, value: &str, path: &Path, line: usize) -> Result<(), String> {
        for raw in split_list(value) {
            let id = RuleId::parse(&raw)
                .ok_or_else(|| format!("{}:{}: unknown rule '{}'", path.display(), line, raw))?;
            self.ignored_rules.insert(id);
        }
        Ok(())
    }
}

fn parse_positive(value: &str, key: &str, path: &Path, line: usize) -> Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| format!("{}:{}: invalid {}", path.display(), line, key))?;
    if parsed == 0 {
        return Err(format!(
            "{}:{}: {} must be greater than zero",
            path.display(),
            line,
            key
        ));
    }
    Ok(parsed)
}

fn unquote(value: &str) -> String {
    value.trim_matches('"').trim_matches('\'').to_string()
}

fn split_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|item| unquote(item.trim()).to_ascii_lowercase())
        .filter(|item| !item.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn fixture(contents: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "englishlint-config-{}.ini",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn parses_limits_ignores_and_glossary() {
        let path = fixture(
            "[lint]\nprocedural_limit = 4\nignore_rules = ENG003\n[glossary]\ncheck = verify\n",
        );
        let config = Config::read(&path).unwrap();
        assert_eq!(config.procedural_limit, 4);
        assert!(config.ignored_rules.contains(&RuleId::BannedModal));
        assert_eq!(config.glossary["check"], "verify");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn rejects_unknown_rule_and_glossary() {
        for contents in [
            "[lint]\nignore_rules = ENG999\n",
            "[glossary]\nunknown = term\n",
        ] {
            let path = fixture(contents);
            assert!(Config::read(&path).is_err());
            let _ = fs::remove_file(path);
        }
    }
}
