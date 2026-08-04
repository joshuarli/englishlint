use crate::diagnostic::Severity;
use crate::rules::RuleId;
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Clone, Debug, Default)]
pub struct Profile {
    pub paths: Vec<String>,
    pub ignored_rules: HashSet<RuleId>,
    pub enabled_rules: HashSet<RuleId>,
    pub severity: HashMap<RuleId, Severity>,
    pub ignored_words: HashSet<String>,
}

impl Profile {
    pub fn matches(&self, path: &Path) -> bool {
        let value = path.to_string_lossy().replace('\\', "/");
        self.paths
            .iter()
            .any(|pattern| glob_matches(pattern, &value))
    }
}

fn glob_matches(pattern: &str, path: &str) -> bool {
    let pattern = pattern.trim().trim_matches('/');
    let path = path.trim_matches('/');
    if pattern.is_empty() {
        return false;
    }
    if let Some(prefix) = pattern.strip_suffix("/**") {
        return path == prefix
            || path.starts_with(&format!("{prefix}/"))
            || path.contains(&format!("/{prefix}/"));
    }
    let candidates = [
        path,
        path.rsplit_once('/').map_or(path, |(_, suffix)| suffix),
    ];
    candidates.iter().any(|candidate| {
        glob_match(pattern, candidate) || candidate.ends_with(&format!("/{pattern}"))
    })
}

fn glob_match(pattern: &str, value: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let value: Vec<char> = value.chars().collect();
    let mut table = vec![vec![false; value.len() + 1]; pattern.len() + 1];
    table[0][0] = true;
    for i in 0..pattern.len() {
        for j in 0..=value.len() {
            if !table[i][j] {
                continue;
            }
            match pattern[i] {
                '*' => {
                    table[i + 1][j] = true;
                    if j < value.len() {
                        table[i][j + 1] = true;
                    }
                }
                '?' if j < value.len() => table[i + 1][j + 1] = true,
                character if j < value.len() && character == value[j] => table[i + 1][j + 1] = true,
                _ => {}
            }
        }
    }
    table[pattern.len()][value.len()]
}

pub(crate) fn apply_profile(
    diagnostics: &mut Vec<crate::diagnostic::Diagnostic>,
    profile: &Profile,
) {
    diagnostics.retain(|diagnostic| {
        profile.enabled_rules.is_empty() || profile.enabled_rules.contains(&diagnostic.rule)
    });
    diagnostics.retain(|diagnostic| !profile.ignored_rules.contains(&diagnostic.rule));
    for diagnostic in diagnostics.iter_mut() {
        if let Some(severity) = profile.severity.get(&diagnostic.rule) {
            diagnostic.severity = *severity;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_file_and_recursive_patterns() {
        assert!(glob_matches("README.md", "/work/README.md"));
        assert!(glob_matches("docs/**", "/work/docs/guide/setup.md"));
        assert!(!glob_matches("docs/**", "/work/roles/engineer.md"));
    }
}
