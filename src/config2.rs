use std::fmt;

use serde::{de::Visitor, Deserialize};
use serde_yaml::{Number, Result as YamlResult};

#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PromptConfig {
    pub prompts: Vec<PromptItem>,

    #[serde(default = "default_entry_dir")]
    pub entry_dir: String,
}

fn default_entry_dir() -> String {
    "{{ repo_name }}".to_owned()
}

impl PromptKind {
    pub fn from_yaml(s: &str) -> YamlResult<Self> {
        serde_yaml::from_str(s).into()
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
        struct LiteralTrueVisitor;

        impl<'de> Visitor<'de> for LiteralTrueVisitor {
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
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct LiteralFalseVisitor;

        impl<'de> Visitor<'de> for LiteralFalseVisitor {
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

        deserializer.deserialize_bool(LiteralFalseVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                })),
            }
        );
    }
}
