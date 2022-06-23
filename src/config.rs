use std::collections::BTreeMap;

use serde::Deserialize;

use crate::prompt::Prompt;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Config {
    #[serde(default, rename(deserialize = "meta"))]
    pub meta_config: MetaConfig,
    #[serde(default)]
    pub prompts: BTreeMap<String, Prompt>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct MetaConfig {
    #[serde(default = "default_prompt_message_for_project_name")]
    pub project_prompt: String,
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
            project_prompt: default_prompt_message_for_project_name(),
            project_var_name: default_project_var_name(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{literal_value::LiteralTrue, prompt::*};

    #[test]
    fn test_deserialize_config() {
        let config = r#"
[meta]
project_prompt = "what's your project name?"
project_var_name = "project"
"#;
        let parsed = toml::from_str::<Config>(config).unwrap();
        assert_eq!(
            parsed,
            Config {
                meta_config: MetaConfig {
                    project_prompt: "what's your project name?".into(),
                    project_var_name: "project".into()
                },
                prompts: BTreeMap::new(),
            }
        );
    }

    #[test]
    fn test_deserialize_empty_config() {
        let config = "";
        let parsed = toml::from_str::<Config>(config).unwrap();
        assert_eq!(
            parsed,
            Config {
                meta_config: MetaConfig {
                    project_prompt: "project name?".into(),
                    project_var_name: "project_name".into()
                },
                prompts: BTreeMap::new(),
            }
        )
    }

    #[test]
    fn test_deserialize_config_with_prompts() {
        let config = r#"
[meta]
project_prompt = "what's your project name?"
project_var_name = "project"

[prompts.name]
prompt = "what's your name?"
type = "string"

[prompts.age]
prompt = "what's your age?"
type = "number"
max = 150

[prompts.love_rust]
prompt = "do you love rust?"
type = "bool"
default = true

[prompts.hobbies]
prompt = "what's your hobbies?"
type = "string"
choices = ["swimming", "running", "reading"]
multi = true

[prompts.country]
prompt = "what's your nationality?"
type = "string"
choices = ["Chinese", "American", "Japanese"]
"#;
        let parsed = toml::from_str::<Config>(config).unwrap();
        assert_eq!(
            parsed,
            Config {
                meta_config: MetaConfig {
                    project_prompt: "what's your project name?".into(),
                    project_var_name: "project".into()
                },
                prompts: [
                    (
                        "name".into(),
                        Prompt {
                            prompt: Some("what's your name?".into()),
                            kind: PromptKind::String(StringPrompt::Normal {
                                default: None,
                                regex: None,
                            })
                        }
                    ),
                    (
                        "age".into(),
                        Prompt {
                            prompt: Some("what's your age?".into()),
                            kind: PromptKind::Number(NumberPrompt::Normal {
                                default: None,
                                max: Some(150_f64),
                                min: None,
                            })
                        }
                    ),
                    (
                        "love_rust".into(),
                        Prompt {
                            prompt: Some("do you love rust?".into()),
                            kind: PromptKind::Bool(BoolPrompt { default: true })
                        }
                    ),
                    (
                        "hobbies".into(),
                        Prompt {
                            prompt: Some("what's your hobbies?".into()),
                            kind: PromptKind::String(StringPrompt::MultiSelector {
                                choices: vec![
                                    "swimming".into(),
                                    "running".into(),
                                    "reading".into()
                                ],
                                default: None,
                                multi: LiteralTrue,
                            })
                        }
                    ),
                    (
                        "country".into(),
                        Prompt {
                            prompt: Some("what's your nationality?".into()),
                            kind: PromptKind::String(StringPrompt::SingleSelector {
                                choices: vec![
                                    "Chinese".into(),
                                    "American".into(),
                                    "Japanese".into()
                                ],
                                default: None,
                            })
                        }
                    ),
                ]
                .into_iter()
                .collect(),
            }
        )
    }
}
