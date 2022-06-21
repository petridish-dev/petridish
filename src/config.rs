use serde::Deserialize;

use crate::prompt::Prompt;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Config {
    #[serde(default, rename(deserialize = ".meta"))]
    pub meta_config: MetaConfig,
    #[serde(default)]
    pub prompts: Vec<Prompt>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct MetaConfig {
    #[serde(default = "default_prompt_message_for_project_name")]
    pub project_prompt_message: String,
    #[serde(default = "default_project_var_name")]
    pub project_var_name: String,
}

fn default_prompt_message_for_project_name() -> String {
    "project name?".into()
}

fn default_project_var_name() -> String {
    "project_name".into()
}

impl Default for MetaConfig {
    fn default() -> Self {
        Self {
            project_prompt_message: default_prompt_message_for_project_name(),
            project_var_name: default_project_var_name(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use serde_test::{assert_de_tokens, Token};

    use super::*;
    use crate::prompt::*;

    #[test]
    fn test_deserialize_config() {
        let config = "\
---
.meta:
  project_prompt_message: what's your project name?
  project_var_name: project
";
        let parsed = serde_yaml::from_str::<Config>(config).unwrap();
        assert_de_tokens(
            &parsed,
            &[
                Token::Struct {
                    name: "Config",
                    len: 1,
                },
                Token::Str(".meta"),
                Token::Struct {
                    name: "MetaConfig",
                    len: 2,
                },
                Token::Str("project_prompt_message"),
                Token::Str("what's your project name?"),
                Token::Str("project_var_name"),
                Token::Str("project"),
                Token::StructEnd,
                Token::StructEnd,
            ],
        )
    }

    #[test]
    fn test_deserialize_empty_config() {
        let config = "{}";
        let parsed = serde_yaml::from_str::<Config>(config).unwrap();
        assert_de_tokens(
            &parsed,
            &[
                Token::Struct {
                    name: "Config",
                    len: 1,
                },
                Token::Str(".meta"),
                Token::Struct {
                    name: "MetaConfig",
                    len: 2,
                },
                Token::Str("project_prompt_message"),
                Token::Str("project name?"),
                Token::Str("project_var_name"),
                Token::Str("project_name"),
                Token::StructEnd,
                Token::StructEnd,
            ],
        )
    }

    #[test]
    fn test_deserialize_config_with_prompts() {
        let config = "\
---
prompts:
- choices: [1, 2, 3]
  name: A
- choices: [a, b, c]
  default: [b]
  name: B
- confirm: true
  name: C
- name: D
  default: hello
";
        let parsed = serde_yaml::from_str::<Config>(config).unwrap();
        assert_eq!(
            parsed,
            Config {
                meta_config: MetaConfig::default(),
                prompts: vec![
                    Prompt {
                        name: "A".into(),
                        prompt_message: None,
                        kind: PromptKind::SingleSelector(SingleSelector::Number(
                            SingleSelectorConfig {
                                default: None,
                                choices: vec![1_f64, 2_f64, 3_f64],
                                multi: None,
                            }
                        ))
                    },
                    Prompt {
                        name: "B".into(),
                        prompt_message: None,
                        kind: PromptKind::MultiSelector(MultiSelector::String(
                            MultiSelectorConfig {
                                default: Some(vec!["b".into()]),
                                choices: vec!["a".into(), "b".into(), "c".into()],
                                multi: None,
                                emptyable: false
                            }
                        ))
                    },
                    Prompt {
                        name: "C".into(),
                        prompt_message: None,
                        kind: PromptKind::Confirm {
                            confirm: LiteralTrue,
                            default: false,
                        }
                    },
                    Prompt {
                        name: "D".into(),
                        prompt_message: None,
                        kind: PromptKind::Literal {
                            default: LiteralValue::String("hello".into()),
                        }
                    }
                ]
            }
        )
    }
}
