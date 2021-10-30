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
    default: Option<String>,
    #[serde(default)]
    choices: Option<Vec<String>>,
    #[serde(default)]
    multi: Option<bool>,
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
  default: Peter
  choices: [Peter, Alice, Joe]
  multi: true
"#;
        assert_eq!(
            serde_yaml::from_str::<PromptConfig>(config).unwrap(),
            PromptConfig {
                prompts: vec![PromptItem {
                    name: "name".to_string(),
                    default: Some("Peter".to_string()),
                    choices: Some(vec![
                        "Peter".to_string(),
                        "Alice".to_string(),
                        "Joe".to_string()
                    ]),
                    multi: Some(true),
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
  multi: true
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
                    default: None,
                    choices: None,
                    multi: None,
                }]
            }
        )
    }
}
