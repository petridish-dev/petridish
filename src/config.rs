use serde::{de::Visitor, Deserialize};

#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PromptConfig {
    pub prompts: Vec<PromptItem>,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PromptTypeName {
    Int,
    Float,
    Boolean,
    String,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum PromptType {
    Int(i32),
    Float(f32),
    Boolean(bool),
    String(String),
}

#[derive(Debug, PartialEq)]
pub struct MultiChoice;

struct MultiChoiceVisitor;
impl<'de> Visitor<'de> for MultiChoiceVisitor {
    type Value = MultiChoice;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("boolean")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v {
            Ok(MultiChoice {})
        } else {
            Err(E::custom(format!("must be boolean `true`")))
        }
    }
}

impl<'de> Deserialize<'de> for MultiChoice {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_bool(MultiChoiceVisitor {})
    }
}

#[derive(Debug, PartialEq)]
pub struct SingleChoice;

struct SingleChoiceVisitor;
impl<'de> Visitor<'de> for SingleChoiceVisitor {
    type Value = SingleChoice;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("boolean")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v {
            Err(E::custom(format!("must be boolean `false`")))
        } else {
            Ok(SingleChoice {})
        }
    }
}

impl<'de> Deserialize<'de> for SingleChoice {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_bool(SingleChoiceVisitor {})
    }
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum PromptItem {
    Normal {
        name: String,
        #[serde(rename = "type")]
        type_: Option<PromptTypeName>,
        default: Option<PromptType>,
    },
    SingleChoice {
        name: String,
        #[serde(rename = "type", default)]
        type_: Option<PromptTypeName>,
        choices: Vec<PromptType>,
        default: Option<PromptType>,
        multi: Option<SingleChoice>,
    },
    MultiChoice {
        name: String,
        #[serde(rename = "type", default)]
        type_: Option<PromptTypeName>,
        choices: Vec<PromptType>,
        default: Option<Vec<PromptType>>,
        multi: MultiChoice,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml;

    #[test]
    fn it_specify_prompt_only_with_name() {
        let yaml = r#"
---
prompts:
- name: name
"#;
        let config = serde_yaml::from_str::<PromptConfig>(yaml).unwrap();
        assert_eq!(
            config.prompts.first().unwrap(),
            &PromptItem::Normal {
                name: "name".to_string(),
                type_: None,
                default: None,
            }
        )
    }

    #[test]
    fn it_prompt_with_type() {
        let yaml = r#"
---
prompts:
- name: age
  type: int
"#;
        let config = serde_yaml::from_str::<PromptConfig>(yaml).unwrap();
        assert_eq!(
            config.prompts.first().unwrap(),
            &PromptItem::Normal {
                name: "age".to_string(),
                type_: Some(PromptTypeName::Int),
                default: None,
            }
        )
    }

    #[test]
    fn it_prompt_missing_name() {
        let yaml = r#"
---
prompts:
- type: int
"#;
        let result = serde_yaml::from_str::<PromptConfig>(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn it_prompt_with_default_value() {
        let yaml = r#"
---
prompts:
- name: age
  default: 18
    "#;
        let config = serde_yaml::from_str::<PromptConfig>(yaml).unwrap();
        assert_eq!(
            config.prompts.first().unwrap(),
            &PromptItem::Normal {
                name: "age".to_string(),
                type_: None,
                default: Some(PromptType::Int(18)),
            }
        )
    }

    #[test]
    fn it_prompt_with_single_choice_v1() {
        let yaml = r#"
---
prompts:
- name: hobby
  choices: [swim, running]
    "#;
        let config = serde_yaml::from_str::<PromptConfig>(yaml).unwrap();
        assert_eq!(
            config.prompts.first().unwrap(),
            &PromptItem::SingleChoice {
                name: "hobby".to_string(),
                type_: None,
                choices: vec![
                    PromptType::String("swim".to_string()),
                    PromptType::String("running".to_string()),
                ],
                default: None,
                multi: None
            }
        )
    }

    #[test]
    fn it_prompt_with_single_choice_v2() {
        let yaml = r#"
---
prompts:
- name: hobby
  choices: [swim, running]
  multi: false
    "#;
        let config = serde_yaml::from_str::<PromptConfig>(yaml).unwrap();
        assert_eq!(
            config.prompts.first().unwrap(),
            &PromptItem::SingleChoice {
                name: "hobby".to_string(),
                type_: None,
                choices: vec![
                    PromptType::String("swim".to_string()),
                    PromptType::String("running".to_string()),
                ],
                default: None,
                multi: Some(SingleChoice {}),
            }
        )
    }

    #[test]
    fn it_prompt_with_single_choice_v3() {
        let yaml = r#"
---
prompts:
- name: age
  choices: [10, 18]
    "#;
        let config = serde_yaml::from_str::<PromptConfig>(yaml).unwrap();
        assert_eq!(
            config.prompts.first().unwrap(),
            &PromptItem::SingleChoice {
                name: "age".to_string(),
                type_: None,
                choices: vec![PromptType::Int(10), PromptType::Int(18)],
                default: None,
                multi: None
            }
        )
    }

    #[test]
    fn it_prompt_with_single_choice_and_default() {
        let yaml = r#"
---
prompts:
- name: age
  choices: [10, 18]
  default: 10
    "#;
        let config = serde_yaml::from_str::<PromptConfig>(yaml).unwrap();
        assert_eq!(
            config.prompts.first().unwrap(),
            &PromptItem::SingleChoice {
                name: "age".to_string(),
                type_: None,
                choices: vec![PromptType::Int(10), PromptType::Int(18)],
                default: Some(PromptType::Int(10)),
                multi: None
            }
        )
    }

    #[test]
    fn it_prompt_with_multi_choice() {
        let yaml = r#"
---
prompts:
- name: hobby
  choices: [swim, running]
  multi: true
    "#;
        let config = serde_yaml::from_str::<PromptConfig>(yaml).unwrap();
        assert_eq!(
            config.prompts.first().unwrap(),
            &PromptItem::MultiChoice {
                name: "hobby".to_string(),
                type_: None,
                choices: vec![
                    PromptType::String("swim".to_string()),
                    PromptType::String("running".to_string())
                ],
                default: None,
                multi: MultiChoice {},
            }
        )
    }

    #[test]
    fn it_prompt_with_multi_choice_and_default() {
        let yaml = r#"
---
prompts:
- name: hobby
  choices: [swim, running]
  multi: true
  default: [swim]
    "#;
        let config = serde_yaml::from_str::<PromptConfig>(yaml).unwrap();
        assert_eq!(
            config.prompts.first().unwrap(),
            &PromptItem::MultiChoice {
                name: "hobby".to_string(),
                type_: None,
                choices: vec![
                    PromptType::String("swim".to_string()),
                    PromptType::String("running".to_string())
                ],
                default: Some(vec![PromptType::String("swim".to_string())]),
                multi: MultiChoice {},
            }
        )
    }

    #[test]
    fn it_prompt_with_multi_choice_and_single_default() {
        let yaml = r#"
---
prompts:
- name: hobby
  choices: [swim, running]
  multi: true
  default: swim
    "#;
        let result = serde_yaml::from_str::<PromptConfig>(yaml);
        assert!(result.is_err());
    }
}
