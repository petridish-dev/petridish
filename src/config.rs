use std::{fmt, fs, path::Path};

use regex::Regex;
use serde::{de::Visitor, Deserialize};
use serde_yaml::Number;

use crate::error::{Error, Result};

#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PromptConfig {
    pub prompts: Vec<PromptItem>,

    #[serde(default = "default_entry_dir")]
    pub entry_dir: String,
    #[serde(default = "default_entry_dir_prompt_message")]
    pub entry_dir_prompt_message: String,
}

fn default_entry_dir() -> String {
    "{{ repo_name }}".to_owned()
}

fn default_entry_dir_prompt_message() -> String {
    "repo dir name?".to_owned()
}

impl PromptConfig {
    pub fn from_yaml(s: &str) -> Result<Self> {
        let config = serde_yaml::from_str::<Self>(s).map_err(Error::ParseError)?;
        for prompt in &config.prompts {
            prompt.validate()?;
        }

        Ok(config)
    }

    pub fn from_yaml_path(p: &Path) -> Result<Self> {
        Self::from_yaml(&fs::read_to_string(p).map_err(Error::Io)?)
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct PromptItem {
    pub name: String,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(flatten)]
    pub kind: PromptKind,
}

impl PromptItem {
    fn validate(&self) -> Result<()> {
        let regex_expression = r"^[a-zA-Z_$][a-zA-Z_$0-9]*$";
        if !Regex::new(regex_expression).unwrap().is_match(&self.name) {
            return Err(Error::ValidateError {
                field: "name".into(),
                error: format!("must match '{}'", regex_expression),
            });
        }

        match &self.kind {
            PromptKind::SingleSelect(t) => match t {
                SingleSelectType::String(v) => {
                    if let Some(default) = &v.default {
                        if !v.choices.contains(default) {
                            return Err(Error::ValidateError {
                                field: "default".into(),
                                error: format!(
                                    "default '{}' is not one of {}",
                                    default,
                                    v.choices
                                        .iter()
                                        .map(|s| format!("'{}'", s))
                                        .collect::<Vec<String>>()
                                        .join(", ")
                                ),
                            });
                        }
                    }
                }
                SingleSelectType::Number(v) => {
                    if let Some(default) = &v.default {
                        if !v.choices.contains(default) {
                            return Err(Error::ValidateError {
                                field: "default".into(),
                                error: format!(
                                    "default '{}' is not one of {}",
                                    default,
                                    v.choices
                                        .iter()
                                        .map(|s| format!("'{}'", s))
                                        .collect::<Vec<String>>()
                                        .join(", ")
                                ),
                            });
                        }
                    }
                }
            },
            PromptKind::MultiSelect(t) => match t {
                MultiSelectType::String(v) => {
                    if let Some(defaults) = &v.default {
                        for default in defaults {
                            if !v.choices.contains(default) {
                                return Err(Error::ValidateError {
                                    field: "default".into(),
                                    error: format!(
                                        "default '{}' is not one of {}",
                                        default,
                                        v.choices
                                            .iter()
                                            .map(|s| format!("'{}'", s))
                                            .collect::<Vec<String>>()
                                            .join(", ")
                                    ),
                                });
                            }
                        }
                    }
                }
                MultiSelectType::Number(v) => {
                    if let Some(defaults) = &v.default {
                        for default in defaults {
                            if !v.choices.contains(default) {
                                return Err(Error::ValidateError {
                                    field: "default".into(),
                                    error: format!(
                                        "default '{}' is not one of {}",
                                        default,
                                        v.choices
                                            .iter()
                                            .map(|s| format!("'{}'", s))
                                            .collect::<Vec<String>>()
                                            .join(", ")
                                    ),
                                });
                            }
                        }
                    }
                }
            },
            _ => (),
        }

        Ok(())
    }
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum PromptKind {
    SingleSelect(SingleSelectType),
    MultiSelect(MultiSelectType),
    Confirm {
        confirm: LiteralTrue,
        #[serde(default)]
        default: bool,
    },
    Default {
        default: Option<Value>,
    },
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
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

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum SingleSelectType {
    String(SingleSelect<String>),
    Number(SingleSelect<Number>),
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum MultiSelectType {
    String(MultiSelect<String>),
    Number(MultiSelect<Number>),
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct SingleSelect<T> {
    pub default: Option<T>,
    pub choices: Vec<T>,
    pub multi: Option<LiteralFalse>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct MultiSelect<T> {
    pub default: Option<Vec<T>>,
    pub choices: Vec<T>,
    pub multi: Option<LiteralTrue>,
    #[serde(default)]
    pub emptyable: bool,
}

#[derive(PartialEq)]
pub struct LiteralTrue;
impl fmt::Debug for LiteralTrue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "true")
    }
}

impl<'de> Deserialize<'de> for LiteralTrue {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct LiteralTrueVisitor;

        impl<'de> Visitor<'de> for LiteralTrueVisitor {
            type Value = LiteralTrue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("bool `true`")
            }

            fn visit_bool<E>(self, v: bool) -> std::result::Result<Self::Value, E>
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

        deserializer.deserialize_bool(LiteralTrueVisitor)
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
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct LiteralFalseVisitor;

        impl<'de> Visitor<'de> for LiteralFalseVisitor {
            type Value = LiteralFalse;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("bool `false`")
            }

            fn visit_bool<E>(self, v: bool) -> std::result::Result<Self::Value, E>
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

        deserializer.deserialize_bool(LiteralFalseVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{io, path::PathBuf};

    use tempdir::TempDir;

    #[test]
    fn only_name() {
        let config = r#"
---
name: your_name
"#;
        let item = serde_yaml::from_str::<PromptItem>(config).unwrap();
        assert_eq!(
            item,
            PromptItem {
                name: "your_name".into(),
                message: None,
                kind: PromptKind::Default { default: None },
            }
        )
    }

    #[test]
    fn validate_prompt_config() {
        let config = r#"
---
prompts:
- name: your-name
"#;
        match PromptConfig::from_yaml(config).err().unwrap() {
            Error::ValidateError { field, error } => {
                assert_eq!(field, "name".to_string());
                assert_eq!(error, "must match '^[a-zA-Z_$][a-zA-Z_$0-9]*$'".to_string());
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn with_message() {
        let config = r#"
---
name: your_name
message: What's your name
"#;
        let item = serde_yaml::from_str::<PromptItem>(config).unwrap();
        assert_eq!(
            item,
            PromptItem {
                name: "your_name".into(),
                message: Some("What's your name".into()),
                kind: PromptKind::Default { default: None },
            }
        )
    }

    #[test]
    fn single_select() {
        let config = r#"
---
name: your_name
choices: [Peter, Alice]
default: Peter
multi: false
"#;
        let item = serde_yaml::from_str::<PromptItem>(config).unwrap();
        assert_eq!(
            item,
            PromptItem {
                name: "your_name".into(),
                message: None,
                kind: PromptKind::SingleSelect(SingleSelectType::String(SingleSelect {
                    default: Some("Peter".into()),
                    choices: vec!["Peter".into(), "Alice".into()],
                    multi: Some(LiteralFalse {}),
                })),
            }
        );

        let config = r#"
---
name: number
choices: [1, 2]
default: 1
multi: false
"#;
        let item = serde_yaml::from_str::<PromptItem>(config).unwrap();
        assert_eq!(
            item,
            PromptItem {
                name: "number".to_string(),
                message: None,
                kind: PromptKind::SingleSelect(SingleSelectType::Number(SingleSelect {
                    default: Some(1.into()),
                    choices: vec![1.into(), 2.into()],
                    multi: Some(LiteralFalse {}),
                })),
            }
        );
    }

    #[test]
    fn validate_single_select() {
        let config = r#"
---
name: your_name
choices: [Peter, Alice]
default: Joe
multi: false
"#;

        match serde_yaml::from_str::<PromptItem>(config)
            .unwrap()
            .validate()
            .err()
            .unwrap()
        {
            Error::ValidateError { field, error } => {
                assert_eq!(field, "default".to_string());
                assert_eq!(
                    error,
                    "default 'Joe' is not one of 'Peter', 'Alice'".to_string()
                );
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn single_select_omit_multi() {
        let config = r#"
---
name: your_name
choices: [Peter, Alice]
default: Peter
"#;
        let item = serde_yaml::from_str::<PromptItem>(config).unwrap();
        assert_eq!(
            item,
            PromptItem {
                name: "your_name".to_string(),
                message: None,
                kind: PromptKind::SingleSelect(SingleSelectType::String(SingleSelect {
                    default: Some("Peter".into()),
                    choices: vec!["Peter".into(), "Alice".into()],
                    multi: None,
                })),
            }
        )
    }

    #[test]
    fn single_select_omit_default() {
        let config = r#"
---
name: your_name
choices: [Peter, Alice]
"#;
        let item = serde_yaml::from_str::<PromptItem>(config).unwrap();
        assert_eq!(
            item,
            PromptItem {
                name: "your_name".into(),
                message: None,
                kind: PromptKind::SingleSelect(SingleSelectType::String(SingleSelect {
                    default: None,
                    choices: vec!["Peter".into(), "Alice".into()],
                    multi: None,
                })),
            }
        )
    }

    #[test]
    fn multi_select() {
        let config = r#"
---
name: your_name
choices: [Peter, Alice]
default: [Peter]
multi: true
"#;
        let item = serde_yaml::from_str::<PromptItem>(config).unwrap();
        assert_eq!(
            item,
            PromptItem {
                name: "your_name".into(),
                message: None,
                kind: PromptKind::MultiSelect(MultiSelectType::String(MultiSelect {
                    default: Some(vec!["Peter".into()]),
                    choices: vec!["Peter".into(), "Alice".into()],
                    multi: Some(LiteralTrue {}),
                    emptyable: false,
                })),
            }
        );

        let config = r#"
---
name: number
choices: [1, 2]
default: [1]
multi: true
"#;
        let item = serde_yaml::from_str::<PromptItem>(config).unwrap();
        assert_eq!(
            item,
            PromptItem {
                name: "number".into(),
                message: None,
                kind: PromptKind::MultiSelect(MultiSelectType::Number(MultiSelect {
                    default: Some(vec![1.into()]),
                    choices: vec![1.into(), 2.into()],
                    multi: Some(LiteralTrue {}),
                    emptyable: false,
                })),
            }
        );
    }

    #[test]
    fn multi_select_omit_default() {
        let config = r#"
---
name: your_name
choices: [Peter, Alice]
multi: true
"#;
        let item = serde_yaml::from_str::<PromptItem>(config).unwrap();
        assert_eq!(
            item,
            PromptItem {
                name: "your_name".into(),
                message: None,
                kind: PromptKind::MultiSelect(MultiSelectType::String(MultiSelect {
                    default: None,
                    choices: vec!["Peter".into(), "Alice".into()],
                    multi: Some(LiteralTrue {}),
                    emptyable: false,
                })),
            }
        );
    }

    #[test]
    fn multi_select_omit_multi() {
        let config = r#"
---
name: your_name
choices: [Peter, Alice]
default: [Peter]
"#;
        let item = serde_yaml::from_str::<PromptItem>(config).unwrap();
        assert_eq!(
            item,
            PromptItem {
                name: "your_name".into(),
                message: None,
                kind: PromptKind::MultiSelect(MultiSelectType::String(MultiSelect {
                    default: Some(vec!["Peter".into()]),
                    choices: vec!["Peter".into(), "Alice".into()],
                    multi: None,
                    emptyable: false,
                })),
            }
        );
    }

    #[test]
    fn validate_multi_select() {
        let config = r#"
---
name: your_name
choices: [Peter, Alice]
default: [Joe]
"#;

        match serde_yaml::from_str::<PromptItem>(config)
            .unwrap()
            .validate()
            .err()
            .unwrap()
        {
            Error::ValidateError { field, error } => {
                assert_eq!(field, "default".to_string());
                assert_eq!(
                    error,
                    "default 'Joe' is not one of 'Peter', 'Alice'".to_string()
                );
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn load_from_yaml_file() {
        let config = r#"
---
prompts:
- name: your_name
"#;
        let tmp_dir = TempDir::new("tmp").unwrap();
        let config_path = &tmp_dir.path().join("petridish.yaml");
        fs::write(config_path, config).unwrap();

        assert_eq!(
            PromptConfig::from_yaml_path(config_path).unwrap(),
            PromptConfig {
                prompts: vec![PromptItem {
                    name: "your_name".into(),
                    message: None,
                    kind: PromptKind::Default { default: None },
                }],
                entry_dir: "{{ repo_name }}".to_string(),
                entry_dir_prompt_message: "repo dir name?".to_string(),
            }
        )
    }

    #[test]
    fn load_missing_yaml_file() {
        match PromptConfig::from_yaml_path(&PathBuf::from("/tmo/a.yaml"))
            .err()
            .unwrap()
        {
            Error::Io(error) => {
                assert_eq!(error.kind(), io::ErrorKind::NotFound);
            }
            _ => unreachable!(),
        }
    }
}
