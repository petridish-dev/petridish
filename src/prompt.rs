use serde::Deserialize;

use crate::{literal_str, literal_value::LiteralTrue};

literal_str!("str", LiteralStr);
literal_str!("number", LiteralNumber);

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Normal {
    String {
        #[serde(rename = "type")]
        type_name: LiteralStr,
        regex: Option<String>,
        default: Option<String>,
    },
    Number {
        #[serde(rename = "type")]
        type_name: LiteralNumber,
        min: Option<f64>,
        max: Option<f64>,
        default: Option<f64>,
    },
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum SingleSelector {
    String {
        #[serde(rename = "type")]
        type_name: LiteralStr,
        choices: Vec<String>,
        default: Option<String>,
    },
    Number {
        #[serde(rename = "type")]
        type_name: LiteralNumber,
        choices: Vec<f64>,
        default: Option<f64>,
    },
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum MultiSelector {
    String {
        #[serde(rename = "type")]
        type_name: LiteralStr,
        choices: Vec<String>,
        default: Option<Vec<String>>,
        multi: LiteralTrue,
    },
    Number {
        #[serde(rename = "type")]
        type_name: LiteralNumber,
        choices: Vec<f64>,
        default: Option<Vec<f64>>,
        multi: LiteralTrue,
    },
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum PromptKind {
    MultiSelector(MultiSelector),
    SingleSelector(SingleSelector),
    Confirm {
        confirm: LiteralTrue,
        #[serde(default)]
        default: bool,
    },
    Normal(Normal),
}

#[derive(Deserialize, Debug, PartialEq)]
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
            type_name: LiteralNumber,
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
            type_name: LiteralNumber,
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
            type_name: LiteralNumber,
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
            type_name: LiteralNumber,
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
            type_name: LiteralNumber,
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
            type_name: LiteralNumber,
            max: Some(2_f64),
            min: Some(1_f64),
            default: Some(1_f64),
        }
    )}}

    test_prompt! {test_literal_str, r#"
prompt="hello"
type="str"
"#, Prompt { 
    prompt: Some("hello".into()),
    kind: PromptKind::Normal(
        Normal::String {
            type_name: LiteralStr,
            regex: None,
            default: None
        }
    )}}

    test_prompt! {test_literal_str_with_default, r#"
prompt="hello"
type="str"
default="abc"
"#, Prompt { 
    prompt: Some("hello".into()),
    kind: PromptKind::Normal(
        Normal::String {
            type_name: LiteralStr,
            regex: None,
            default: Some("abc".into())
        }
    )}}

    test_prompt! {test_literal_str_with_regex, r#"
prompt="hello"
type="str"
regex="abc"
"#, Prompt { 
    prompt: Some("hello".into()),
    kind: PromptKind::Normal(
        Normal::String {
            type_name: LiteralStr,
            regex: Some("abc".into()),
            default: None
        }
    )}}

    test_prompt! {test_literal_str_with_regex_and_default, r#"
prompt="hello"
type="str"
regex="a.*c"
default="abc"
"#, Prompt { 
    prompt: Some("hello".into()),
    kind: PromptKind::Normal(
        Normal::String {
            type_name: LiteralStr,
            regex: Some("a.*c".into()),
            default: Some("abc".into()),
        }
    )}}

    test_prompt! {test_confirm, r#"
prompt="ok?"
confirm=true
"#, Prompt { 
    prompt: Some("ok?".into()),
    kind: PromptKind::Confirm{
        confirm: LiteralTrue,
        default: false
    }}}

    test_prompt! {test_confirm_with_default, r#"
prompt="ok?"
confirm=true
default=true
"#, Prompt { 
    prompt: Some("ok?".into()),
    kind: PromptKind::Confirm{
        confirm: LiteralTrue,
        default: true
    }}}

    test_prompt! {test_number_single_choice, r#"
prompt="age"
choices=[1, 2, 3]
type="number"
"#, Prompt { 
    prompt: Some("age".into()),
    kind: PromptKind::SingleSelector(
        SingleSelector::Number{
            type_name: LiteralNumber,
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
            type_name: LiteralNumber,
            choices: vec![1_f64, 2_f64, 3_f64],
            default: Some(1_f64),
        })
    }}

    test_prompt! {test_str_single_choice, r#"
prompt="name"
choices=["a", "b", "c"]
type="str"
"#, Prompt { 
    prompt: Some("name".into()),
    kind: PromptKind::SingleSelector(
        SingleSelector::String{
            type_name: LiteralStr,
            choices: vec!["a".into(), "b".into(), "c".into()],
            default: None,
        })
    }}

    test_prompt! {test_str_single_choice_with_default, r#"
prompt="name"
choices=["a", "b", "c"]
type="str"
default="a"
"#, Prompt { 
    prompt: Some("name".into()),
    kind: PromptKind::SingleSelector(
        SingleSelector::String{
            type_name: LiteralStr,
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
            type_name: LiteralNumber,
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
            type_name: LiteralNumber,
            choices: vec![1_f64, 2_f64, 3_f64],
            default: Some(vec![1_f64]),
            multi: LiteralTrue,
        })
    }}

    test_prompt! {test_str_multi_choices, r#"
prompt="name"
choices=["a", "b", "c"]
type="str"
multi=true
"#, Prompt { 
    prompt: Some("name".into()),
    kind: PromptKind::MultiSelector(
        MultiSelector::String{
            type_name: LiteralStr,
            choices: vec!["a".into(), "b".into(), "c".into()],
            default: None,
            multi: LiteralTrue,
        })
    }}

    test_prompt! {test_str_multi_choices_with_default, r#"
prompt="name"
choices=["a", "b", "c"]
type="str"
default=["a"]
multi=true
"#, Prompt { 
    prompt: Some("name".into()),
    kind: PromptKind::MultiSelector(
        MultiSelector::String{
            type_name: LiteralStr,
            choices: vec!["a".into(), "b".into(), "c".into()],
            default: Some(vec!["a".into()]),
            multi: LiteralTrue,
        })
    }}
}
