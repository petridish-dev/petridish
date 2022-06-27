use serde::Deserialize;

use crate::prompt::PromptConfig;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Config {
    #[serde(default, rename(deserialize = "petridish"))]
    pub petridish_config: PetridishConfig,
    #[serde(default)]
    pub prompts: Vec<PromptConfig>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct PetridishConfig {
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

impl Default for PetridishConfig {
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
[petridish]
project_prompt = "what's your project name?"
project_var_name = "project"
"#;
        let parsed = toml::from_str::<Config>(config).unwrap();
        assert_eq!(
            parsed,
            Config {
                petridish_config: PetridishConfig {
                    project_prompt: "what's your project name?".into(),
                    project_var_name: "project".into()
                },
                prompts: vec![],
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
                petridish_config: PetridishConfig {
                    project_prompt: "project name?".into(),
                    project_var_name: "project_name".into()
                },
                prompts: vec![],
            }
        )
    }

    #[test]
    fn test_deserialize_config_with_prompts() {
        let config = r#"
[petridish]
project_prompt = "what's your project name?"
project_var_name = "project"

[[prompts]]
name = "name"
prompt = "what's your name?"
type = "string"

[[prompts]]
name = "age"
prompt = "what's your age?"
type = "number"
max = 150

[[prompts]]
name = "love_rust"
prompt = "do you love rust?"
type = "bool"
default = true

[[prompts]]
name = "hobbies"
prompt = "what's your hobbies?"
type = "string"
choices = ["swimming", "running", "reading"]
multi = true

[[prompts]]
name = "nationality"
prompt = "what's your nationality?"
type = "string"
choices = ["Chinese", "American", "Japanese"]
"#;
        let parsed = toml::from_str::<Config>(config).unwrap();
        assert_eq!(
            parsed,
            Config {
                petridish_config: PetridishConfig {
                    project_prompt: "what's your project name?".into(),
                    project_var_name: "project".into()
                },
                prompts: vec![
                    PromptConfig {
                        name: "name".into(),
                        prompt: Some("what's your name?".into()),
                        kind: PromptKind::String(StringPrompt::Normal {
                            default: None,
                            regex: None,
                        })
                    },
                    PromptConfig {
                        name: "age".into(),
                        prompt: Some("what's your age?".into()),
                        kind: PromptKind::Number(NumberPrompt::Normal {
                            default: None,
                            max: Some(150_f64),
                            min: None,
                        })
                    },
                    PromptConfig {
                        name: "love_rust".into(),
                        prompt: Some("do you love rust?".into()),
                        kind: PromptKind::Bool(BoolPrompt { default: true })
                    },
                    PromptConfig {
                        name: "hobbies".into(),
                        prompt: Some("what's your hobbies?".into()),
                        kind: PromptKind::String(StringPrompt::MultiSelector {
                            choices: vec!["swimming".into(), "running".into(), "reading".into()],
                            default: None,
                            multi: LiteralTrue,
                        })
                    },
                    PromptConfig {
                        name: "nationality".into(),
                        prompt: Some("what's your nationality?".into()),
                        kind: PromptKind::String(StringPrompt::SingleSelector {
                            choices: vec!["Chinese".into(), "American".into(), "Japanese".into()],
                            default: None,
                        })
                    }
                ]
            }
        )
    }
}
