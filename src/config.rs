use serde::{Deserialize, Serialize};

use crate::literal_value::LiteralTrue;

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

#[derive(Deserialize, Debug, PartialEq, Serialize)]
pub struct PromptConfig {
    pub name: String,
    pub prompt: Option<String>,
    #[serde(flatten)]
    pub prompt_type: PromptType,
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PromptType {
    String(StringPrompt),
    Number(NumberPrompt),
    Bool(BoolPrompt),
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum StringPrompt {
    MultiSelect {
        multi: LiteralTrue,
        choices: Vec<String>,
        default: Option<Vec<String>>,
    },
    Select {
        choices: Vec<String>,
        default: Option<String>,
    },
    Input {
        default: Option<String>,
        regex: Option<String>,
    },
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum NumberPrompt {
    MultiSelect {
        multi: LiteralTrue,
        choices: Vec<f64>,
        default: Option<Vec<f64>>,
    },
    Select {
        choices: Vec<f64>,
        default: Option<f64>,
    },
    Input {
        default: Option<f64>,
        min: Option<f64>,
        max: Option<f64>,
    },
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
pub struct BoolPrompt {
    #[serde(default)]
    pub default: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_prompt {
        ($dst:ident, $config:literal, $parsed:expr) => {
            #[test]
            fn $dst() {
                let parsed = toml::from_str::<PromptConfig>($config).unwrap();
                assert_eq!(parsed, $parsed);
            }
        };
    }

    test_prompt! {test_literal_number, r#"
name="var"
prompt="hello"
type="number"
"#, PromptConfig { 
    name: "var".into(),
    prompt: Some("hello".into()),
    prompt_type: PromptType::Number(
        NumberPrompt::Input {
            max: None,
            min: None,
            default: None
        }
    )}}

    test_prompt! {test_literal_number_with_default, r#"
name="var"
prompt="hello"
type="number"
default=1
"#, PromptConfig {
    name: "var".into(),
    prompt: Some("hello".into()),
    prompt_type: PromptType::Number(
        NumberPrompt::Input {
            max: None,
            min: None,
            default: Some(1_f64),
        }
    )}}

    test_prompt! {test_literal_number_with_max, r#"
name="var"
prompt="hello"
type="number"
max=1
"#, PromptConfig {
    name: "var".into(),
    prompt: Some("hello".into()),
    prompt_type: PromptType::Number(
        NumberPrompt::Input {
            max: Some(1_f64),
            min: None,
            default: None,
        }
    )}}

    test_prompt! {test_literal_number_with_min, r#"
name="var"
prompt="hello"
type="number"
min=1
"#, PromptConfig {
    name: "var".into(),
    prompt: Some("hello".into()),
    prompt_type: PromptType::Number(
        NumberPrompt::Input {
            max: None,
            min: Some(1_f64),
            default: None,
        }
    )}}

    test_prompt! {test_literal_number_with_min_and_max, r#"
name="var"
prompt="hello"
type="number"
min=1
max=2
"#, PromptConfig {
    name: "var".into(),
    prompt: Some("hello".into()),
    prompt_type: PromptType::Number(
        NumberPrompt::Input {
            max: Some(2_f64),
            min: Some(1_f64),
            default: None,
        }
    )}}

    test_prompt! {test_literal_number_with_min_and_max_and_default, r#"
name="var"
prompt="hello"
type="number"
min=1
max=2
default=1
"#, PromptConfig {
    name: "var".into(),
    prompt: Some("hello".into()),
    prompt_type: PromptType::Number(
        NumberPrompt::Input {
            max: Some(2_f64),
            min: Some(1_f64),
            default: Some(1_f64),
        }
    )}}

    test_prompt! {test_literal_str, r#"
name="var"
prompt="hello"
type="string"
"#, PromptConfig {
    name: "var".into(),
    prompt: Some("hello".into()),
    prompt_type: PromptType::String(
        StringPrompt::Input {
            regex: None,
            default: None
        }
    )}}

    test_prompt! {test_literal_str_with_default, r#"
name="var"
prompt="hello"
type="string"
default="abc"
"#, PromptConfig {
    name: "var".into(),
    prompt: Some("hello".into()),
    prompt_type: PromptType::String(
        StringPrompt::Input {
            regex: None,
            default: Some("abc".into())
        }
    )}}

    test_prompt! {test_literal_str_with_regex, r#"
name="var"
prompt="hello"
type="string"
regex="abc"
"#, PromptConfig {
    name: "var".into(),
    prompt: Some("hello".into()),
    prompt_type: PromptType::String(
        StringPrompt::Input {
            regex: Some("abc".into()),
            default: None
        }
    )}}

    test_prompt! {test_literal_str_with_regex_and_default, r#"
name="var"
prompt="hello"
type="string"
regex="a.*c"
default="abc"
"#, PromptConfig {
    name: "var".into(),
    prompt: Some("hello".into()),
    prompt_type: PromptType::String(
        StringPrompt::Input {
            regex: Some("a.*c".into()),
            default: Some("abc".into()),
        }
    )}}

    test_prompt! {test_confirm, r#"
name="var"    
prompt="ok?"
type="bool"
"#, PromptConfig {
    name: "var".into(),
    prompt: Some("ok?".into()),
    prompt_type: PromptType::Bool(BoolPrompt {default: false})}
    }

    test_prompt! {test_confirm_with_default, r#"
name="var"
prompt="ok?"
type="bool"
default=true
"#, PromptConfig {
    name: "var".into(),
    prompt: Some("ok?".into()),
    prompt_type: PromptType::Bool(BoolPrompt {default: true})}
    }

    test_prompt! {test_number_single_choice, r#"
name="var"
prompt="age"
choices=[1, 2, 3]
type="number"
"#, PromptConfig {
    name: "var".into(),
    prompt: Some("age".into()),
    prompt_type: PromptType::Number(
        NumberPrompt::Select{
            choices: vec![1_f64, 2_f64, 3_f64],
            default: None,
        })
    }}

    test_prompt! {test_number_single_choice_with_default, r#"
name="var"
prompt="age"
choices=[1, 2, 3]
type="number"
default=1
"#, PromptConfig {
    name: "var".into(),
    prompt: Some("age".into()),
    prompt_type: PromptType::Number(
        NumberPrompt::Select{
            choices: vec![1_f64, 2_f64, 3_f64],
            default: Some(1_f64),
        })
    }}

    test_prompt! {test_str_single_choice, r#"
name="var"
prompt="name"
choices=["a", "b", "c"]
type="string"
"#, PromptConfig {
    name: "var".into(),
    prompt: Some("name".into()),
    prompt_type: PromptType::String(
        StringPrompt::Select{
            choices: vec!["a".into(), "b".into(), "c".into()],
            default: None,
        })
    }}

    test_prompt! {test_str_single_choice_with_default, r#"
name="var"
prompt="name"
choices=["a", "b", "c"]
type="string"
default="a"
"#, PromptConfig {
    name: "var".into(),
    prompt: Some("name".into()),
    prompt_type: PromptType::String(
        StringPrompt::Select{
            choices: vec!["a".into(), "b".into(), "c".into()],
            default: Some("a".into()),
        })
    }}

    test_prompt! {test_number_multi_choices, r#"
name="var"
prompt="age"
choices=[1, 2, 3]
type="number"
multi=true
"#, PromptConfig {
    name: "var".into(),
    prompt: Some("age".into()),
    prompt_type: PromptType::Number(
        NumberPrompt::MultiSelect{
            choices: vec![1_f64, 2_f64, 3_f64],
            default: None,
            multi: LiteralTrue,
        })
    }}

    test_prompt! {test_number_multi_choices_with_default, r#"
name="var"
prompt="age"
choices=[1, 2, 3]
type="number"
default=[1]
multi=true
"#, PromptConfig {
    name: "var".into(),
    prompt: Some("age".into()),
    prompt_type: PromptType::Number(
        NumberPrompt::MultiSelect{
            choices: vec![1_f64, 2_f64, 3_f64],
            default: Some(vec![1_f64]),
            multi: LiteralTrue,
        })
    }}

    test_prompt! {test_str_multi_choices, r#"
name="var"
prompt="name"
choices=["a", "b", "c"]
type="string"
multi=true
"#, PromptConfig {
    name: "var".into(),
    prompt: Some("name".into()),
    prompt_type: PromptType::String(
        StringPrompt::MultiSelect{
            choices: vec!["a".into(), "b".into(), "c".into()],
            default: None,
            multi: LiteralTrue,
        })
    }}

    test_prompt! {test_str_multi_choices_with_default, r#"
name="var"    
prompt="name"
choices=["a", "b", "c"]
type="string"
default=["a"]
multi=true
"#, PromptConfig {
    name: "var".into(),
    prompt: Some("name".into()),
    prompt_type: PromptType::String(
        StringPrompt::MultiSelect{
            choices: vec!["a".into(), "b".into(), "c".into()],
            default: Some(vec!["a".into()]),
            multi: LiteralTrue,
        })
    }}

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
                        prompt_type: PromptType::String(StringPrompt::Input {
                            default: None,
                            regex: None,
                        })
                    },
                    PromptConfig {
                        name: "age".into(),
                        prompt: Some("what's your age?".into()),
                        prompt_type: PromptType::Number(NumberPrompt::Input {
                            default: None,
                            max: Some(150_f64),
                            min: None,
                        })
                    },
                    PromptConfig {
                        name: "love_rust".into(),
                        prompt: Some("do you love rust?".into()),
                        prompt_type: PromptType::Bool(BoolPrompt { default: true })
                    },
                    PromptConfig {
                        name: "hobbies".into(),
                        prompt: Some("what's your hobbies?".into()),
                        prompt_type: PromptType::String(StringPrompt::MultiSelect {
                            choices: vec!["swimming".into(), "running".into(), "reading".into()],
                            default: None,
                            multi: LiteralTrue,
                        })
                    },
                    PromptConfig {
                        name: "nationality".into(),
                        prompt: Some("what's your nationality?".into()),
                        prompt_type: PromptType::String(StringPrompt::Select {
                            choices: vec!["Chinese".into(), "American".into(), "Japanese".into()],
                            default: None,
                        })
                    }
                ]
            }
        )
    }
}
