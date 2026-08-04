use crate::error::ConfigError;
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
    pub max_file_bytes: usize,
    pub profiles: HashMap<String, crate::policy::Profile>,
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
            max_file_bytes: 10 * 1024 * 1024,
            profiles: HashMap::new(),
        }
    }
}

impl Config {
    pub fn profile_for(&self, path: &Path) -> crate::policy::Profile {
        self.profile_named(path, None)
    }

    pub fn profile_named(&self, path: &Path, name: Option<&str>) -> crate::policy::Profile {
        let mut profile = crate::policy::Profile::default();
        for candidate in self
            .profiles
            .iter()
            .filter(|(candidate_name, profile)| {
                profile.matches(path) && name.is_none_or(|wanted| wanted == candidate_name.as_str())
            })
            .map(|(_, profile)| profile)
        {
            profile
                .ignored_rules
                .extend(candidate.ignored_rules.iter().copied());
            profile
                .enabled_rules
                .extend(candidate.enabled_rules.iter().copied());
            profile
                .ignored_words
                .extend(candidate.ignored_words.iter().cloned());
            profile.severity.extend(
                candidate
                    .severity
                    .iter()
                    .map(|(rule, severity)| (*rule, *severity)),
            );
        }
        profile
    }

    pub(crate) fn with_profile(&self, profile: &crate::policy::Profile) -> Self {
        let mut effective = self.clone();
        effective
            .ignored_rules
            .extend(profile.ignored_rules.iter().copied());
        effective
            .ignored_words
            .extend(profile.ignored_words.iter().cloned());
        effective
    }

    pub fn read(path: &Path) -> Result<Self, ConfigError> {
        let mut config = Self::default();
        if !path.exists() {
            return Ok(config);
        }
        let source = fs::read_to_string(path).map_err(|source| ConfigError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let mut section = String::new();
        for (line_index, raw) in source.lines().enumerate() {
            let line_number = line_index + 1;
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = line[1..line.len() - 1].trim().to_ascii_lowercase();
                continue;
            }
            let Some(equal) = line.find('=') else {
                return Err(ConfigError::InvalidSyntax {
                    path: path.to_path_buf(),
                    line: line_number,
                    message: "expected section or key = value".into(),
                });
            };
            let key = line[..equal].trim().to_ascii_lowercase();
            let value = line[equal + 1..].trim();
            match section.as_str() {
                "lint" => config.set_lint(&key, value, path, line_number)?,
                "ignore" => config.set_ignore(&key, value, path, line_number)?,
                section if section.starts_with("profile.") => config.set_profile(
                    section.trim_start_matches("profile."),
                    &key,
                    value,
                    path,
                    line_number,
                )?,
                "glossary" => {
                    if !matches!(key.as_str(), "check" | "config" | "delete") {
                        return Err(ConfigError::UnknownGlossaryConcept {
                            path: path.to_path_buf(),
                            line: line_number,
                            concept: key,
                        });
                    }
                    config.glossary.insert(key, unquote(value));
                }
                _ => {
                    return Err(ConfigError::UnknownSection {
                        path: path.to_path_buf(),
                        line: line_number,
                        section,
                    })
                }
            }
        }
        Ok(config)
    }

    fn set_lint(
        &mut self,
        key: &str,
        value: &str,
        path: &Path,
        line: usize,
    ) -> Result<(), ConfigError> {
        match key {
            "default_type" => {
                self.default_mode = TextMode::parse(value).ok_or_else(|| {
                    invalid(path, line, key, value, "must be procedural or descriptive")
                })?
            }
            "descriptive_limit" => self.descriptive_limit = parse_positive(value, key, path, line)?,
            "procedural_limit" => self.procedural_limit = parse_positive(value, key, path, line)?,
            "procedural_headings" => self.procedural_headings = split_list(value),
            "ignore_rules" => self.add_rules(value, path, line)?,
            "ignore_words" => self.ignored_words.extend(split_list(value)),
            "max_file_bytes" => self.max_file_bytes = parse_positive(value, key, path, line)?,
            _ => {
                return Err(ConfigError::UnknownKey {
                    path: path.to_path_buf(),
                    line,
                    section: "lint".into(),
                    key: key.into(),
                })
            }
        }
        Ok(())
    }

    fn set_profile(
        &mut self,
        name: &str,
        key: &str,
        value: &str,
        path: &Path,
        line: usize,
    ) -> Result<(), ConfigError> {
        let profile = self.profiles.entry(name.to_string()).or_default();
        match key {
            "paths" | "include" => profile.paths = split_list(value),
            "ignore_rules" => add_profile_rules(&mut profile.ignored_rules, value, path, line)?,
            "enable_rules" | "rules" => {
                add_profile_rules(&mut profile.enabled_rules, value, path, line)?
            }
            "ignore_words" => profile.ignored_words.extend(split_list(value)),
            "severity" => {
                for item in split_list(value) {
                    let Some((raw_rule, raw_severity)) = item.split_once(':') else {
                        return Err(invalid(
                            path,
                            line,
                            key,
                            value,
                            "use ENG001:error,ENG014:warning",
                        ));
                    };
                    let rule = RuleId::parse(raw_rule).ok_or_else(|| ConfigError::UnknownRule {
                        path: path.to_path_buf(),
                        line,
                        value: raw_rule.into(),
                    })?;
                    let severity = match raw_severity {
                        "error" => crate::diagnostic::Severity::Error,
                        "warning" => crate::diagnostic::Severity::Warning,
                        "info" => crate::diagnostic::Severity::Info,
                        _ => {
                            return Err(invalid(
                                path,
                                line,
                                key,
                                raw_severity,
                                "must be error, warning, or info",
                            ))
                        }
                    };
                    profile.severity.insert(rule, severity);
                }
            }
            _ => {
                return Err(ConfigError::UnknownKey {
                    path: path.to_path_buf(),
                    line,
                    section: format!("profile.{name}"),
                    key: key.into(),
                })
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
    ) -> Result<(), ConfigError> {
        match key {
            "rules" => self.add_rules(value, path, line)?,
            "words" => self.ignored_words.extend(split_list(value)),
            _ => {
                return Err(ConfigError::UnknownKey {
                    path: path.to_path_buf(),
                    line,
                    section: "ignore".into(),
                    key: key.into(),
                })
            }
        }
        Ok(())
    }

    fn add_rules(&mut self, value: &str, path: &Path, line: usize) -> Result<(), ConfigError> {
        for raw in split_list(value) {
            let id = RuleId::parse(&raw).ok_or_else(|| ConfigError::UnknownRule {
                path: path.to_path_buf(),
                line,
                value: raw.clone(),
            })?;
            self.ignored_rules.insert(id);
        }
        Ok(())
    }
}

fn add_profile_rules(
    target: &mut HashSet<RuleId>,
    value: &str,
    path: &Path,
    line: usize,
) -> Result<(), ConfigError> {
    for raw in split_list(value) {
        let rule = RuleId::parse(&raw).ok_or_else(|| ConfigError::UnknownRule {
            path: path.to_path_buf(),
            line,
            value: raw.clone(),
        })?;
        target.insert(rule);
    }
    Ok(())
}

fn invalid(path: &Path, line: usize, key: &str, value: &str, message: &str) -> ConfigError {
    ConfigError::InvalidValue {
        path: path.to_path_buf(),
        line,
        key: key.into(),
        value: value.into(),
        message: message.into(),
    }
}

fn parse_positive(value: &str, key: &str, path: &Path, line: usize) -> Result<usize, ConfigError> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| invalid(path, line, key, value, "must be a positive integer"))?;
    if parsed == 0 {
        return Err(invalid(path, line, key, value, "must be greater than zero"));
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
    fn parses_profiles_and_severity() {
        let path = fixture("[profile.guidance]\npaths = docs/**, README.md\nignore_rules = ENG014\nseverity = ENG001:error,ENG003:warning\n");
        let config = Config::read(&path).unwrap();
        let profile = config.profile_for(Path::new("docs/guide.md"));
        assert!(profile.ignored_rules.contains(&RuleId::PassiveVoice));
        assert_eq!(
            profile.severity[&RuleId::BannedModal],
            crate::diagnostic::Severity::Warning
        );
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
