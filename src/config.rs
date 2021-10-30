use regex::{self, Regex};
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PromptConfig {
    pub prompts: Vec<PromptItem>,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PromptItem {
    name: String,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    default: Option<String>,
    #[serde(default)]
    choices: Option<Vec<String>>,
}

impl PromptConfig {
    pub fn from_yaml(s: &str) -> Result<PromptConfig, String> {
        let config = serde_yaml::from_str::<PromptConfig>(s).map_err(|e| e.to_string())?;
        for (idx, prompt_item) in config.prompts.iter().enumerate() {
            prompt_item
                .validate()
                .map_err(|e| format!("prompts[{}]: {}", idx, e))?
        }

        Ok(config)
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
                }]
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
                }]
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
}
