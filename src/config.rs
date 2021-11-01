use std::fs;

use miette::Diagnostic;
use regex::{self, Regex};
use serde::Deserialize;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum ConfigError {
    #[error("{0}")]
    ParseError(String),

    #[error("prompts[{index}]: {err}")]
    CustomParseError { index: usize, err: String },

    #[error("{0}")]
    LoadConfigFailed(String),
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PromptConfig {
    pub prompts: Vec<PromptItem>,

    #[serde(default = "default_entry_dir")]
    pub entry_dir: String,
}

fn default_entry_dir() -> String {
    "{{ repo_name }}".to_string()
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PromptItem {
    pub name: String,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub choices: Option<Vec<String>>,
}

impl PromptConfig {
    pub fn from_yaml(s: &str) -> Result<PromptConfig, ConfigError> {
        let config = serde_yaml::from_str::<PromptConfig>(s)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;
        for (idx, prompt_item) in config.prompts.iter().enumerate() {
            prompt_item
                .validate()
                .map_err(|e| ConfigError::CustomParseError { index: idx, err: e })?
        }

        Ok(config)
    }

    pub fn from_yaml_path(p: &Path) -> Result<PromptConfig, ConfigError> {
        let content =
            fs::read_to_string(p).map_err(|e| ConfigError::LoadConfigFailed(e.to_string()))?;

        Ok(PromptConfig::from_yaml(&content)?)
    }
}

impl PromptItem {
    fn validate(&self) -> Result<(), String> {
        let regex_expression = r"^[a-zA-Z_$][a-zA-Z_$0-9]*$";
        if !Regex::new(regex_expression).unwrap().is_match(&self.name) {
            Err(format!("name must match `{}`", regex_expression))?
        }

        match (&self.default, &self.choices) {
            (Some(default), Some(choices)) => {
                if !choices.contains(default) {
                    Err(format!(
                        "invalid default, `{}` is not one of {}",
                        default,
                        choices
                            .iter()
                            .map(|s| format!("`{}`", s))
                            .collect::<Vec<String>>()
                            .join(", ")
                    ))?
                }
            }
            _ => (),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use serde_yaml;
    use tempdir::TempDir;

    #[test]
    fn it_deserialize_prompt_config() {
        let config = r#"
---
prompts:
- name: name
  message: What's your name
  default: Peter
  choices: [Peter, Alice, Joe]
"#;
        assert_eq!(
            serde_yaml::from_str::<PromptConfig>(config).unwrap(),
            PromptConfig {
                prompts: vec![PromptItem {
                    name: "name".to_string(),
                    message: Some("What's your name".to_string()),
                    default: Some("Peter".to_string()),
                    choices: Some(vec![
                        "Peter".to_string(),
                        "Alice".to_string(),
                        "Joe".to_string()
                    ]),
                }],
                entry_dir: "{{ repo_name }}".to_string(),
            }
        )
    }

    #[test]
    fn it_deserialize_prompt_config_missing_name() {
        let config = r#"
---
prompts:
- default: Peter
  choices: [Peter, Alice, Joe]
"#;
        match serde_yaml::from_str::<PromptConfig>(config) {
            Ok(_) => unreachable!(),
            Err(err) => {
                assert_eq!(
                    err.to_string(),
                    "prompts[0]: missing field `name` at line 4 column 10"
                )
            }
        }
    }

    #[test]
    fn it_deserialize_prompt_config_omit_fields() {
        let config = r#"
---
prompts:
- name: name
"#;
        assert_eq!(
            serde_yaml::from_str::<PromptConfig>(config).unwrap(),
            PromptConfig {
                prompts: vec![PromptItem {
                    name: "name".to_string(),
                    message: None,
                    default: None,
                    choices: None,
                }],
                entry_dir: "{{ repo_name }}".to_string(),
            }
        )
    }

    #[test]
    fn it_validate_name() {
        let config = r#"
---
prompts:
- name: 1_name
"#;

        match PromptConfig::from_yaml(config) {
            Ok(_) => unreachable!(),
            Err(err) => {
                assert_eq!(
                    err.to_string(),
                    "prompts[0]: name must match `^[a-zA-Z_$][a-zA-Z_$0-9]*$`"
                )
            }
        }
    }

    #[test]
    fn it_validate_choices_with_default() {
        let config = r#"
---
prompts:
- name: name
  choices: [a, b]
  default: c
"#;

        match PromptConfig::from_yaml(config) {
            Ok(_) => unreachable!(),
            Err(err) => {
                assert_eq!(
                    err.to_string(),
                    "prompts[0]: invalid default, `c` is not one of `a`, `b`"
                )
            }
        }
    }

    #[test]
    fn it_deserialize_prompt_config_from_file() {
        let config = r#"
---
prompts:
- name: name
"#;
        let tmp_dir = TempDir::new("tmp").unwrap();
        let config_path = &tmp_dir.path().join("petridish.yaml");
        fs::write(config_path, config).unwrap();

        assert_eq!(
            PromptConfig::from_yaml_path(config_path).unwrap(),
            PromptConfig {
                prompts: vec![PromptItem {
                    name: "name".to_string(),
                    message: None,
                    default: None,
                    choices: None,
                }],
                entry_dir: "{{ repo_name }}".to_string(),
            }
        )
    }
}
