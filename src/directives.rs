use crate::config::Config;
use crate::rules::RuleId;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Default)]
pub struct DirectiveSet {
    file: HashSet<RuleId>,
    lines: HashMap<usize, HashSet<RuleId>>,
    pub words: HashSet<String>,
}

impl DirectiveSet {
    pub fn parse(source: &str) -> Self {
        let mut result = Self::default();
        for (line_index, line) in source.lines().enumerate() {
            let lower = line.to_ascii_lowercase();
            let Some(start) = lower.find("<!-- englishlint:") else {
                continue;
            };
            let command = lower[start + "<!-- englishlint:".len()..]
                .split("-->")
                .next()
                .unwrap_or("")
                .trim();
            let mut fields = command.split_whitespace();
            let Some(action) = fields.next() else {
                continue;
            };
            let values: Vec<&str> = fields.collect();
            match action {
                "ignore-file" => result.add_rules(&values, None),
                "ignore" | "ignore-next-line" => result.add_rules(&values, Some(line_index + 2)),
                "ignore-line" => result.add_rules(&values, Some(line_index + 1)),
                "ignore-word" => result.words.extend(
                    values
                        .into_iter()
                        .map(|word| word.trim_matches(',').to_string()),
                ),
                _ => {}
            }
        }
        result
    }

    fn add_rules(&mut self, values: &[&str], line: Option<usize>) {
        let parsed = values
            .iter()
            .filter_map(|value| RuleId::parse(value.trim_matches(',')));
        match line {
            Some(line) => self.lines.entry(line).or_default().extend(parsed),
            None => self.file.extend(parsed),
        }
    }

    pub fn suppresses(&self, rule: RuleId, line: usize, config: &Config) -> bool {
        config.ignored_rules.contains(&rule)
            || self.file.contains(&rule)
            || self
                .lines
                .get(&line)
                .is_some_and(|rules| rules.contains(&rule))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_file_line_and_word_suppressions() {
        let set = DirectiveSet::parse("<!-- englishlint: ignore-file ENG003 -->\n<!-- englishlint: ignore-next-line ENG004 -->\n<!-- englishlint: ignore-word widget -->\n");
        assert!(set.file.contains(&RuleId::BannedModal));
        assert!(set.lines.get(&3).unwrap().contains(&RuleId::Semicolon));
        assert!(set.words.contains("widget"));
    }
}
