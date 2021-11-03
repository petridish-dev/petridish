use std::fmt;
use std::fs;
use std::path::Path;

use miette::Diagnostic;
use regex::{self, Regex};
use serde::de::Visitor;
use serde::Deserialize;
use serde_yaml::Number;
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
pub struct PromptItem {
    pub name: String,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default, flatten)]
    pub kind: PromptKind,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum PromptKind {
    SingleChoice {
        default: Option<Value>,
        choices: Vec<Value>,
        multi: Option<LiteralFalse>,
    },
    MultiChoices {
        default: Option<Vec<Value>>,
        choices: Vec<Value>,
        multi: Option<LiteralTrue>,
    },
    Flag {
        flag: LiteralTrue,
        #[serde(default)]
        default: bool,
    },
    Normal {
        default: Option<Value>,
    },
}

impl Default for PromptKind {
    fn default() -> Self {
        Self::Normal { default: None }
    }
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Value {
    Number(Number),
    String(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(v) => {
                write!(f, "{}", v)
            }
            Value::String(v) => {
                write!(f, "{}", v)
            }
        }
    }
}

#[derive(PartialEq)]
pub struct LiteralTrue;

impl fmt::Debug for LiteralTrue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "true")
    }
}

impl<'de> Deserialize<'de> for LiteralTrue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct MultiChoicesVisitor;

        impl<'de> Visitor<'de> for MultiChoicesVisitor {
            type Value = LiteralTrue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("bool `true`")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v {
                    Ok(LiteralTrue {})
                } else {
                    Err(E::custom("must be bool true"))
                }
            }
        }

        deserializer.deserialize_bool(MultiChoicesVisitor)
    }
}

#[derive(PartialEq)]
pub struct LiteralFalse;

impl fmt::Debug for LiteralFalse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "false")
    }
}

impl<'de> Deserialize<'de> for LiteralFalse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SingleChoiceVisitor;

        impl<'de> Visitor<'de> for SingleChoiceVisitor {
            type Value = LiteralFalse;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("bool `false`")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if !v {
                    Ok(LiteralFalse {})
                } else {
                    Err(E::custom("must be bool false"))
                }
            }
        }

        deserializer.deserialize_bool(SingleChoiceVisitor)
    }
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

        if let PromptKind::SingleChoice {
            default: Some(default),
            choices,
            multi: _,
        } = &self.kind
        {
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
        } else if let PromptKind::MultiChoices {
            default: Some(default),
            choices,
            multi: _,
        } = &self.kind
        {
            for default_item in default {
                if !choices.contains(default_item) {
                    Err(format!(
                        "invalid default, `{}` is not one of {}",
                        default_item,
                        choices
                            .iter()
                            .map(|s| format!("`{}`", s))
                            .collect::<Vec<String>>()
                            .join(", ")
                    ))?
                }
            }
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
"#;
        assert_eq!(
            serde_yaml::from_str::<PromptConfig>(config).unwrap(),
            PromptConfig {
                prompts: vec![PromptItem {
                    name: "name".to_string(),
                    message: Some("What's your name".to_string()),
                    kind: PromptKind::Normal {
                        default: Some(Value::String("Peter".to_string()))
                    },
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
                    kind: PromptKind::Normal { default: None }
                }],
                entry_dir: "{{ repo_name }}".to_string(),
            }
        )
    }

    #[test]
    fn it_single_choice_v1() {
        let config = r#"
---
prompts:
- name: name
  default: Peter
  choices: [Peter, Alice, Joe]
"#;
        assert_eq!(
            serde_yaml::from_str::<PromptConfig>(config).unwrap(),
            PromptConfig {
                prompts: vec![PromptItem {
                    name: "name".to_string(),
                    message: None,
                    kind: PromptKind::SingleChoice {
                        default: Some(Value::String("Peter".to_string())),
                        choices: vec![
                            Value::String("Peter".to_string()),
                            Value::String("Alice".to_string()),
                            Value::String("Joe".to_string()),
                        ],
                        multi: None,
                    },
                }],
                entry_dir: "{{ repo_name }}".to_string(),
            }
        )
    }

    #[test]
    fn it_single_choice_v2() {
        let config = r#"
---
prompts:
- name: name
  default: Peter
  choices: [Peter, Alice, Joe]
  multi: false
"#;
        assert_eq!(
            serde_yaml::from_str::<PromptConfig>(config).unwrap(),
            PromptConfig {
                prompts: vec![PromptItem {
                    name: "name".to_string(),
                    message: None,
                    kind: PromptKind::SingleChoice {
                        default: Some(Value::String("Peter".to_string())),
                        choices: vec![
                            Value::String("Peter".to_string()),
                            Value::String("Alice".to_string()),
                            Value::String("Joe".to_string()),
                        ],
                        multi: Some(LiteralFalse {}),
                    },
                }],
                entry_dir: "{{ repo_name }}".to_string(),
            }
        )
    }

    #[test]
    fn it_validate_single_choice() {
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
    fn it_multi_choices_v1() {
        let config = r#"
---
prompts:
- name: name
  choices: [a, b]
  multi: true
"#;
        assert_eq!(
            serde_yaml::from_str::<PromptConfig>(config).unwrap(),
            PromptConfig {
                prompts: vec![PromptItem {
                    name: "name".to_string(),
                    message: None,
                    kind: PromptKind::MultiChoices {
                        default: None,
                        choices: vec![
                            Value::String("a".to_string()),
                            Value::String("b".to_string())
                        ],
                        multi: Some(LiteralTrue {})
                    }
                }],
                entry_dir: "{{ repo_name }}".to_string(),
            }
        )
    }

    #[test]
    fn it_multi_choices_v2() {
        let config = r#"
---
prompts:
- name: name
  choices: [a, b]
  default: [a]
"#;
        assert_eq!(
            serde_yaml::from_str::<PromptConfig>(config).unwrap(),
            PromptConfig {
                prompts: vec![PromptItem {
                    name: "name".to_string(),
                    message: None,
                    kind: PromptKind::MultiChoices {
                        default: Some(vec![Value::String("a".to_string())]),
                        choices: vec![
                            Value::String("a".to_string()),
                            Value::String("b".to_string())
                        ],
                        multi: None,
                    }
                }],
                entry_dir: "{{ repo_name }}".to_string(),
            }
        )
    }

    #[test]
    fn it_validate_multi_choices() {
        let config = r#"
---
prompts:
- name: name
  choices: [a, b]
  default: [c, d]
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
                    kind: PromptKind::Normal { default: None },
                }],
                entry_dir: "{{ repo_name }}".to_string(),
            }
        )
    }

    #[test]
    fn it_deserialize_flag_v1() {
        let config = r#"
---
prompts:
- name: name
  flag: true
"#;

        assert_eq!(
            PromptConfig::from_yaml(config).unwrap(),
            PromptConfig {
                prompts: vec![PromptItem {
                    name: "name".to_string(),
                    message: None,
                    kind: PromptKind::Flag {
                        flag: LiteralTrue {},
                        default: false
                    },
                }],
                entry_dir: "{{ repo_name }}".to_string(),
            }
        )
    }

    #[test]
    fn it_deserialize_flag_v2() {
        let config = r#"
---
prompts:
- name: name
  flag: false
"#;

        assert_eq!(
            PromptConfig::from_yaml(config).unwrap(),
            PromptConfig {
                prompts: vec![PromptItem {
                    name: "name".to_string(),
                    message: None,
                    kind: PromptKind::Normal { default: None },
                }],
                entry_dir: "{{ repo_name }}".to_string(),
            }
        )
    }
}
