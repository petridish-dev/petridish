use serde::{Deserialize, Serialize};

use crate::literal_value::LiteralTrue;

#[derive(Deserialize, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Normal {
    String {
        regex: Option<String>,
        default: Option<String>,
    },
    Number {
        min: Option<f64>,
        max: Option<f64>,
        default: Option<f64>,
    },
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SingleSelector {
    String {
        choices: Vec<String>,
        default: Option<String>,
    },
    Number {
        choices: Vec<f64>,
        default: Option<f64>,
    },
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MultiSelector {
    String {
        choices: Vec<String>,
        default: Option<Vec<String>>,
        multi: LiteralTrue,
    },
    Number {
        choices: Vec<f64>,
        default: Option<Vec<f64>>,
        multi: LiteralTrue,
    },
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
#[serde(tag = "type")]
pub struct Confirm {
    #[serde(default)]
    default: bool,
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum PromptKind {
    MultiSelector(MultiSelector),
    SingleSelector(SingleSelector),
    Normal(Normal),
    Confirm(Confirm),
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
pub struct Prompt {
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
prompt="hello"
type="number"
"#, Prompt { 
    prompt: Some("hello".into()),
    kind: PromptKind::Normal(
        Normal::Number {
            max: None,
            min: None,
            default: None
        }
    )}}

    test_prompt! {test_literal_number_with_default, r#"
prompt="hello"
type="number"
default=1
"#, Prompt {
    prompt: Some("hello".into()),
    kind: PromptKind::Normal(
        Normal::Number {
            max: None,
            min: None,
            default: Some(1_f64),
        }
    )}}

    test_prompt! {test_literal_number_with_max, r#"
prompt="hello"
type="number"
max=1
"#, Prompt {
    prompt: Some("hello".into()),
    kind: PromptKind::Normal(
        Normal::Number {
            max: Some(1_f64),
            min: None,
            default: None,
        }
    )}}

    test_prompt! {test_literal_number_with_min, r#"
prompt="hello"
type="number"
min=1
"#, Prompt {
    prompt: Some("hello".into()),
    kind: PromptKind::Normal(
        Normal::Number {
            max: None,
            min: Some(1_f64),
            default: None,
        }
    )}}

    test_prompt! {test_literal_number_with_min_and_max, r#"
prompt="hello"
type="number"
min=1
max=2
"#, Prompt {
    prompt: Some("hello".into()),
    kind: PromptKind::Normal(
        Normal::Number {
            max: Some(2_f64),
            min: Some(1_f64),
            default: None,
        }
    )}}

    test_prompt! {test_literal_number_with_min_and_max_and_default, r#"
prompt="hello"
type="number"
min=1
max=2
default=1
"#, Prompt {
    prompt: Some("hello".into()),
    kind: PromptKind::Normal(
        Normal::Number {
            max: Some(2_f64),
            min: Some(1_f64),
            default: Some(1_f64),
        }
    )}}

    test_prompt! {test_literal_str, r#"
prompt="hello"
type="string"
"#, Prompt {
    prompt: Some("hello".into()),
    kind: PromptKind::Normal(
        Normal::String {
            regex: None,
            default: None
        }
    )}}

    test_prompt! {test_literal_str_with_default, r#"
prompt="hello"
type="string"
default="abc"
"#, Prompt {
    prompt: Some("hello".into()),
    kind: PromptKind::Normal(
        Normal::String {
            regex: None,
            default: Some("abc".into())
        }
    )}}

    test_prompt! {test_literal_str_with_regex, r#"
prompt="hello"
type="string"
regex="abc"
"#, Prompt {
    prompt: Some("hello".into()),
    kind: PromptKind::Normal(
        Normal::String {
            regex: Some("abc".into()),
            default: None
        }
    )}}

    test_prompt! {test_literal_str_with_regex_and_default, r#"
prompt="hello"
type="string"
regex="a.*c"
default="abc"
"#, Prompt {
    prompt: Some("hello".into()),
    kind: PromptKind::Normal(
        Normal::String {
            regex: Some("a.*c".into()),
            default: Some("abc".into()),
        }
    )}}

    test_prompt! {test_confirm, r#"
prompt="ok?"
type="bool"
"#, Prompt {
    prompt: Some("ok?".into()),
    kind: PromptKind::Confirm(Confirm{default: false})}}

    test_prompt! {test_confirm_with_default, r#"
prompt="ok?"
type="bool"
default=true
"#, Prompt {
    prompt: Some("ok?".into()),
    kind: PromptKind::Confirm(Confirm{default: true})}}

    test_prompt! {test_number_single_choice, r#"
prompt="age"
choices=[1, 2, 3]
type="number"
"#, Prompt {
    prompt: Some("age".into()),
    kind: PromptKind::SingleSelector(
        SingleSelector::Number{
            choices: vec![1_f64, 2_f64, 3_f64],
            default: None,
        })
    }}

    test_prompt! {test_number_single_choice_with_default, r#"
prompt="age"
choices=[1, 2, 3]
type="number"
default=1
"#, Prompt {
    prompt: Some("age".into()),
    kind: PromptKind::SingleSelector(
        SingleSelector::Number{
            choices: vec![1_f64, 2_f64, 3_f64],
            default: Some(1_f64),
        })
    }}

    test_prompt! {test_str_single_choice, r#"
prompt="name"
choices=["a", "b", "c"]
type="string"
"#, Prompt {
    prompt: Some("name".into()),
    kind: PromptKind::SingleSelector(
        SingleSelector::String{
            choices: vec!["a".into(), "b".into(), "c".into()],
            default: None,
        })
    }}

    test_prompt! {test_str_single_choice_with_default, r#"
prompt="name"
choices=["a", "b", "c"]
type="string"
default="a"
"#, Prompt {
    prompt: Some("name".into()),
    kind: PromptKind::SingleSelector(
        SingleSelector::String{
            choices: vec!["a".into(), "b".into(), "c".into()],
            default: Some("a".into()),
        })
    }}

    test_prompt! {test_number_multi_choices, r#"
prompt="age"
choices=[1, 2, 3]
type="number"
multi=true
"#, Prompt {
    prompt: Some("age".into()),
    kind: PromptKind::MultiSelector(
        MultiSelector::Number{
            choices: vec![1_f64, 2_f64, 3_f64],
            default: None,
            multi: LiteralTrue,
        })
    }}

    test_prompt! {test_number_multi_choices_with_default, r#"
prompt="age"
choices=[1, 2, 3]
type="number"
default=[1]
multi=true
"#, Prompt {
    prompt: Some("age".into()),
    kind: PromptKind::MultiSelector(
        MultiSelector::Number{
            choices: vec![1_f64, 2_f64, 3_f64],
            default: Some(vec![1_f64]),
            multi: LiteralTrue,
        })
    }}

    test_prompt! {test_str_multi_choices, r#"
prompt="name"
choices=["a", "b", "c"]
type="string"
multi=true
"#, Prompt {
    prompt: Some("name".into()),
    kind: PromptKind::MultiSelector(
        MultiSelector::String{
            choices: vec!["a".into(), "b".into(), "c".into()],
            default: None,
            multi: LiteralTrue,
        })
    }}

    test_prompt! {test_str_multi_choices_with_default, r#"
prompt="name"
choices=["a", "b", "c"]
type="string"
default=["a"]
multi=true
"#, Prompt {
    prompt: Some("name".into()),
    kind: PromptKind::MultiSelector(
        MultiSelector::String{
            choices: vec!["a".into(), "b".into(), "c".into()],
            default: Some(vec!["a".into()]),
            multi: LiteralTrue,
        })
    }}
}
