use serde::{Deserialize, Serialize};

use crate::literal_value::LiteralTrue;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Config {
    #[serde(default, rename(deserialize = "petridish"))]
    pub petridish_config: PetridishConfig,
    #[serde(default)]
    pub prompts: Vec<PromptType>,
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

#[derive(Deserialize, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PromptType {
    String(StringPrompt),
    Number(NumberPrompt),
    Bool(BoolPrompt),
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum StringPrompt {
    MultiSelect(MultiSelect<String>),
    Select(Select<String>),
    Input(StringInput),
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum NumberPrompt {
    MultiSelect(MultiSelect<f64>),
    Select(Select<f64>),
    Input(NumberInput),
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum BoolPrompt {
    Confirm(Confirm),
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct StringInput {
    pub name: String,
    pub prompt: Option<String>,
    pub default: Option<String>,
    pub regex: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct NumberInput {
    pub name: String,
    pub prompt: Option<String>,
    pub default: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Select<T> {
    pub name: String,
    pub prompt: Option<String>,
    pub choices: Vec<T>,
    pub default: Option<T>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct MultiSelect<T> {
    pub multi: LiteralTrue,
    pub name: String,
    pub prompt: Option<String>,
    pub choices: Vec<T>,
    pub default: Option<Vec<T>>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Confirm {
    pub name: String,
    pub prompt: Option<String>,
    #[serde(default)]
    pub default: bool,
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn test_literal_number() {
        let config = r#"
        name="var"
        prompt="hello"
        type="number"        
        "#;
        let parsed = toml::from_str::<PromptType>(config).unwrap();
        let expected = PromptType::Number(NumberPrompt::Input(NumberInput {
            name: "var".into(),
            prompt: Some("hello".into()),
            max: None,
            min: None,
            default: None,
        }));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_literal_number_with_default() {
        let config = r#"
        name="var"
        prompt="hello"
        type="number"        
        default=1
        "#;
        let parsed = toml::from_str::<PromptType>(config).unwrap();
        let expected = PromptType::Number(NumberPrompt::Input(NumberInput {
            name: "var".into(),
            prompt: Some("hello".into()),
            max: None,
            min: None,
            default: Some(1_f64),
        }));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_literal_number_with_min_and_max() {
        let config = r#"
        name="var"
        prompt="hello"
        type="number"        
        min=1
        max=20
        "#;
        let parsed = toml::from_str::<PromptType>(config).unwrap();
        let expected = PromptType::Number(NumberPrompt::Input(NumberInput {
            name: "var".into(),
            prompt: Some("hello".into()),
            min: Some(1_f64),
            max: Some(20_f64),
            default: None,
        }));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_literal_string() {
        let config = r#"
        name="var"
        prompt="hello"
        type="string"
        "#;
        let parsed = toml::from_str::<PromptType>(config).unwrap();
        let expected = PromptType::String(StringPrompt::Input(StringInput {
            name: "var".into(),
            prompt: Some("hello".into()),
            regex: None,
            default: None,
        }));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_literal_string_with_default() {
        let config = r#"
        name="var"
        prompt="hello"
        type="string"
        default="rust"
        "#;
        let parsed = toml::from_str::<PromptType>(config).unwrap();
        let expected = PromptType::String(StringPrompt::Input(StringInput {
            name: "var".into(),
            prompt: Some("hello".into()),
            regex: None,
            default: Some("rust".into()),
        }));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_literal_string_with_regex() {
        let config = r#"
        name="var"
        prompt="hello"
        type="string"
        regex=".*"
        "#;
        let parsed = toml::from_str::<PromptType>(config).unwrap();
        let expected = PromptType::String(StringPrompt::Input(StringInput {
            name: "var".into(),
            prompt: Some("hello".into()),
            regex: Some(".*".into()),
            default: None,
        }));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_confirm() {
        let config = r#"
        name="var"
        prompt="ok?"
        type="bool"
        "#;
        let parsed = toml::from_str::<PromptType>(config).unwrap();
        let expected = PromptType::Bool(BoolPrompt::Confirm(Confirm {
            name: "var".into(),
            prompt: Some("ok?".into()),
            default: false,
        }));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_confirm_with_default() {
        let config = r#"
        name="var"
        prompt="ok?"
        type="bool"
        default=true
        "#;
        let parsed = toml::from_str::<PromptType>(config).unwrap();
        let expected = PromptType::Bool(BoolPrompt::Confirm(Confirm {
            name: "var".into(),
            prompt: Some("ok?".into()),
            default: true,
        }));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_number_select() {
        let config = r#"
        name="var"
        prompt="age"
        choices=[10, 20, 30]
        type="number"
        "#;
        let parsed = toml::from_str::<PromptType>(config).unwrap();
        let expected = PromptType::Number(NumberPrompt::Select(Select {
            name: "var".into(),
            prompt: Some("age".into()),
            choices: vec![10_f64, 20_f64, 30_f64],
            default: None,
        }));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_number_select_with_default() {
        let config = r#"
        name="var"
        prompt="age"
        choices=[10, 20, 30]
        type="number"
        default=10
        "#;
        let parsed = toml::from_str::<PromptType>(config).unwrap();
        let expected = PromptType::Number(NumberPrompt::Select(Select {
            name: "var".into(),
            prompt: Some("age".into()),
            choices: vec![10_f64, 20_f64, 30_f64],
            default: Some(10_f64),
        }));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_string_select() {
        let config = r#"
        name="var"
        prompt="name"
        choices=["a", "b", "c"]
        type="string"
        "#;
        let parsed = toml::from_str::<PromptType>(config).unwrap();
        let expected = PromptType::String(StringPrompt::Select(Select {
            name: "var".into(),
            prompt: Some("name".into()),
            choices: vec!["a".into(), "b".into(), "c".into()],
            default: None,
        }));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_string_select_with_default() {
        let config = r#"
        name="var"
        prompt="name"
        choices=["a", "b", "c"]
        type="string"
        default="a"
        "#;
        let parsed = toml::from_str::<PromptType>(config).unwrap();
        let expected = PromptType::String(StringPrompt::Select(Select {
            name: "var".into(),
            prompt: Some("name".into()),
            choices: vec!["a".into(), "b".into(), "c".into()],
            default: Some("a".into()),
        }));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_number_multi_select() {
        let config = r#"
        name="var"
        prompt="age"
        choices=[10, 20, 30]
        type="number"
        multi=true
        "#;
        let parsed = toml::from_str::<PromptType>(config).unwrap();
        let expected = PromptType::Number(NumberPrompt::MultiSelect(MultiSelect {
            multi: LiteralTrue,
            name: "var".into(),
            prompt: Some("age".into()),
            choices: vec![10_f64, 20_f64, 30_f64],
            default: None,
        }));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_number_multi_select_with_default() {
        let config = r#"
        name="var"
        prompt="age"
        choices=[10, 20, 30]
        type="number"
        multi=true
        default=[10]
        "#;
        let parsed = toml::from_str::<PromptType>(config).unwrap();
        let expected = PromptType::Number(NumberPrompt::MultiSelect(MultiSelect {
            multi: LiteralTrue,
            name: "var".into(),
            prompt: Some("age".into()),
            choices: vec![10_f64, 20_f64, 30_f64],
            default: Some(vec![10_f64]),
        }));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_string_multi_select() {
        let config = r#"
        name="var"
        prompt="name"
        choices=["a", "b", "c"]
        type="string"
        multi=true
        "#;
        let parsed = toml::from_str::<PromptType>(config).unwrap();
        let expected = PromptType::String(StringPrompt::MultiSelect(MultiSelect {
            multi: LiteralTrue,
            name: "var".into(),
            prompt: Some("name".into()),
            choices: vec!["a".into(), "b".into(), "c".into()],
            default: None,
        }));
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_string_multi_select_with_default() {
        let config = r#"
        name="var"
        prompt="name"
        choices=["a", "b", "c"]
        type="string"
        multi=true
        default=["a"]
        "#;
        let parsed = toml::from_str::<PromptType>(config).unwrap();
        let expected = PromptType::String(StringPrompt::MultiSelect(MultiSelect {
            multi: LiteralTrue,
            name: "var".into(),
            prompt: Some("name".into()),
            choices: vec!["a".into(), "b".into(), "c".into()],
            default: Some(vec!["a".into()]),
        }));
        assert_eq!(parsed, expected);
    }

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
                    PromptType::String(StringPrompt::Input(StringInput {
                        name: "name".into(),
                        prompt: Some("what's your name?".into()),
                        default: None,
                        regex: None,
                    })),
                    PromptType::Number(NumberPrompt::Input(NumberInput {
                        name: "age".into(),
                        prompt: Some("what's your age?".into()),
                        default: None,
                        max: Some(150_f64),
                        min: None,
                    })),
                    PromptType::Bool(BoolPrompt::Confirm(Confirm {
                        name: "love_rust".into(),
                        prompt: Some("do you love rust?".into()),
                        default: true,
                    })),
                    PromptType::String(StringPrompt::MultiSelect(MultiSelect {
                        name: "hobbies".into(),
                        prompt: Some("what's your hobbies?".into()),
                        choices: vec!["swimming".into(), "running".into(), "reading".into()],
                        default: None,
                        multi: LiteralTrue,
                    })),
                    PromptType::String(StringPrompt::Select(Select {
                        name: "nationality".into(),
                        prompt: Some("what's your nationality?".into()),
                        choices: vec!["Chinese".into(), "American".into(), "Japanese".into()],
                        default: None,
                    })),
                ]
            }
        )
    }
}
