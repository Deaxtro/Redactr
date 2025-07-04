use serde::Deserialize;
use std::collections::HashMap;
use tracing::{info,debug};

const JSON: &str = include_str!("../regex_rules.json");

#[derive(Deserialize)]
struct JSONRuleConfig {
    pattern: String,
    placeholder: String,
    comment: String,
}

pub struct Rule {
    pub pattern: String,
    pub mapping: HashMap<String, String>,
    pub count: usize,
    pub placeholder: String,
    pub comment: String,
}

impl Rule {
    fn new(pattern: String, placeholder: String, comment: String) -> Self {
        Rule {
            pattern,
            mapping: HashMap::new(),
            count: 0,
            placeholder,
            comment,
        }
    }

    pub fn on_match(&mut self, matched_text: &str) -> String {
        debug!("Redacting: {}", matched_text);
        let redacted_match = self
            .mapping
            .entry(matched_text.to_string())
            .or_insert_with(|| {
                self.count+=1;
                format!("{}{}", self.placeholder, self.count)
            });
        info!("After redacting: {}", redacted_match);
        redacted_match.clone()
    }

}

pub fn load_rule_configs() -> Vec<Rule> {
    let loaded_json: Vec<JSONRuleConfig> = serde_json::from_str(JSON).unwrap();

    let mut rules: Vec<Rule> = Vec::new();
    for rule in loaded_json {
        rules.push(Rule::new(
            rule.pattern.to_string(),
            rule.placeholder,
            rule.comment
        ));
    }
    rules
}