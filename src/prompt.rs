use serde::{Deserialize, Serialize};

use crate::literal_value::LiteralTrue;

#[derive(Deserialize, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PromptKind {
    String(StringPrompt),
    Number(NumberPrompt),
    Bool(BoolPrompt),
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum StringPrompt {
    MultiSelector {
        multi: LiteralTrue,
        choices: Vec<String>,
        default: Option<Vec<String>>,
    },
    SingleSelector {
        choices: Vec<String>,
        default: Option<String>,
    },
    Normal {
        default: Option<String>,
        regex: Option<String>,
    },
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum NumberPrompt {
    MultiSelector {
        multi: LiteralTrue,
        choices: Vec<f64>,
        default: Option<Vec<f64>>,
    },
    SingleSelector {
        choices: Vec<f64>,
        default: Option<f64>,
    },
    Normal {
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

#[derive(Deserialize, Debug, PartialEq, Serialize)]
pub struct Prompt {
    pub name: String,
    pub prompt: Option<String>,
    #[serde(flatten)]
    pub kind: PromptKind,
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_prompt {
        ($dst:ident, $config:literal, $parsed:expr) => {
            #[test]
            fn $dst() {
                let parsed = toml::from_str::<Prompt>($config).unwrap();
                assert_eq!(parsed, $parsed);
            }
        };
    }

    test_prompt! {test_literal_number, r#"
name="var"
prompt="hello"
type="number"
"#, Prompt { 
    name: "var".into(),
    prompt: Some("hello".into()),
    kind: PromptKind::Number(
        NumberPrompt::Normal {
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
"#, Prompt {
    name: "var".into(),
    prompt: Some("hello".into()),
    kind: PromptKind::Number(
        NumberPrompt::Normal {
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
"#, Prompt {
    name: "var".into(),
    prompt: Some("hello".into()),
    kind: PromptKind::Number(
        NumberPrompt::Normal {
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
"#, Prompt {
    name: "var".into(),
    prompt: Some("hello".into()),
    kind: PromptKind::Number(
        NumberPrompt::Normal {
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
"#, Prompt {
    name: "var".into(),
    prompt: Some("hello".into()),
    kind: PromptKind::Number(
        NumberPrompt::Normal {
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
"#, Prompt {
    name: "var".into(),
    prompt: Some("hello".into()),
    kind: PromptKind::Number(
        NumberPrompt::Normal {
            max: Some(2_f64),
            min: Some(1_f64),
            default: Some(1_f64),
        }
    )}}

    test_prompt! {test_literal_str, r#"
name="var"
prompt="hello"
type="string"
"#, Prompt {
    name: "var".into(),
    prompt: Some("hello".into()),
    kind: PromptKind::String(
        StringPrompt::Normal {
            regex: None,
            default: None
        }
    )}}

    test_prompt! {test_literal_str_with_default, r#"
name="var"
prompt="hello"
type="string"
default="abc"
"#, Prompt {
    name: "var".into(),
    prompt: Some("hello".into()),
    kind: PromptKind::String(
        StringPrompt::Normal {
            regex: None,
            default: Some("abc".into())
        }
    )}}

    test_prompt! {test_literal_str_with_regex, r#"
name="var"
prompt="hello"
type="string"
regex="abc"
"#, Prompt {
    name: "var".into(),
    prompt: Some("hello".into()),
    kind: PromptKind::String(
        StringPrompt::Normal {
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
"#, Prompt {
    name: "var".into(),
    prompt: Some("hello".into()),
    kind: PromptKind::String(
        StringPrompt::Normal {
            regex: Some("a.*c".into()),
            default: Some("abc".into()),
        }
    )}}

    test_prompt! {test_confirm, r#"
name="var"    
prompt="ok?"
type="bool"
"#, Prompt {
    name: "var".into(),
    prompt: Some("ok?".into()),
    kind: PromptKind::Bool(BoolPrompt {default: false})}
    }

    test_prompt! {test_confirm_with_default, r#"
name="var"
prompt="ok?"
type="bool"
default=true
"#, Prompt {
    name: "var".into(),
    prompt: Some("ok?".into()),
    kind: PromptKind::Bool(BoolPrompt {default: true})}
    }

    test_prompt! {test_number_single_choice, r#"
name="var"
prompt="age"
choices=[1, 2, 3]
type="number"
"#, Prompt {
    name: "var".into(),
    prompt: Some("age".into()),
    kind: PromptKind::Number(
        NumberPrompt::SingleSelector{
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
"#, Prompt {
    name: "var".into(),
    prompt: Some("age".into()),
    kind: PromptKind::Number(
        NumberPrompt::SingleSelector{
            choices: vec![1_f64, 2_f64, 3_f64],
            default: Some(1_f64),
        })
    }}

    test_prompt! {test_str_single_choice, r#"
name="var"
prompt="name"
choices=["a", "b", "c"]
type="string"
"#, Prompt {
    name: "var".into(),
    prompt: Some("name".into()),
    kind: PromptKind::String(
        StringPrompt::SingleSelector{
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
"#, Prompt {
    name: "var".into(),
    prompt: Some("name".into()),
    kind: PromptKind::String(
        StringPrompt::SingleSelector{
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
"#, Prompt {
    name: "var".into(),
    prompt: Some("age".into()),
    kind: PromptKind::Number(
        NumberPrompt::MultiSelector{
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
"#, Prompt {
    name: "var".into(),
    prompt: Some("age".into()),
    kind: PromptKind::Number(
        NumberPrompt::MultiSelector{
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
"#, Prompt {
    name: "var".into(),
    prompt: Some("name".into()),
    kind: PromptKind::String(
        StringPrompt::MultiSelector{
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
"#, Prompt {
    name: "var".into(),
    prompt: Some("name".into()),
    kind: PromptKind::String(
        StringPrompt::MultiSelector{
            choices: vec!["a".into(), "b".into(), "c".into()],
            default: Some(vec!["a".into()]),
            multi: LiteralTrue,
        })
    }}
}
