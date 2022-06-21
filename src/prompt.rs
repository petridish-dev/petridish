use serde::{de::Visitor, Deserialize};

macro_rules! literal_bool {
    ($src:literal, $dst:ident) => {
        #[derive(PartialEq, Eq)]
        pub struct $dst;

        impl std::fmt::Debug for $dst {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", $src)
            }
        }

        impl<'de> Deserialize<'de> for $dst {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct LiteralVisitor;

                impl<'de> Visitor<'de> for LiteralVisitor {
                    type Value = $dst;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str(&format!("bool `{}`", $src))
                    }

                    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        if v == $src {
                            Ok($dst)
                        } else {
                            Err(E::custom(format!("must be bool `{}`", $src)))
                        }
                    }
                }

                deserializer.deserialize_bool(LiteralVisitor)
            }
        }
    };
}

literal_bool!(true, LiteralTrue);
literal_bool!(false, LiteralFalse);

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum SingleSelector {
    String(SingleSelectorConfig<String>),
    Number(SingleSelectorConfig<f64>),
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct SingleSelectorConfig<T> {
    pub default: Option<T>,
    pub choices: Vec<T>,
    pub multi: Option<LiteralFalse>,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum MultiSelector {
    String(MultiSelectorConfig<String>),
    Number(MultiSelectorConfig<f64>),
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct MultiSelectorConfig<T> {
    pub default: Option<Vec<T>>,
    pub choices: Vec<T>,
    pub multi: Option<LiteralTrue>,
    #[serde(default)]
    pub emptyable: bool,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum PromptKind {
    SingleSelector(SingleSelector),
    MultiSelector(MultiSelector),
    Confirm {
        confirm: LiteralTrue,
        #[serde(default)]
        default: bool,
    },
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_literal_true() {
        assert_eq!(format!("{:?}", LiteralTrue), "true");
        assert_eq!(
            serde_yaml::from_str::<LiteralTrue>("true").unwrap(),
            LiteralTrue
        );
        assert!(serde_yaml::from_str::<LiteralTrue>("false").is_err())
    }

    #[test]
    fn test_literal_false() {
        assert_eq!(format!("{:?}", LiteralFalse), "false");
        assert_eq!(
            serde_yaml::from_str::<LiteralFalse>("false").unwrap(),
            LiteralFalse
        );
        assert!(serde_yaml::from_str::<LiteralFalse>("true").is_err())
    }

    #[test]
    fn test_single_selector() {
        let config = "\
---
default: 1
choices: [1, 2, 3]
";
        let parsed = serde_yaml::from_str::<SingleSelector>(config).unwrap();
        assert_eq!(
            parsed,
            SingleSelector::Number(SingleSelectorConfig {
                default: Some(1_f64),
                choices: vec![1_f64, 2_f64, 3_f64],
                multi: None,
            })
        )
    }

    #[test]
    fn test_single_selector_with_multi_field() {
        let config = "\
---
default: 1
choices: [1, 2, 3]
multi: false
";
        let parsed = serde_yaml::from_str::<SingleSelector>(config).unwrap();
        assert_eq!(
            parsed,
            SingleSelector::Number(SingleSelectorConfig {
                default: Some(1_f64),
                choices: vec![1_f64, 2_f64, 3_f64],
                multi: Some(LiteralFalse),
            })
        )
    }

    #[test]
    fn test_single_selector_for_string() {
        let config = "\
---
default: a
choices: [a, b, c]
";
        let parsed = serde_yaml::from_str::<SingleSelector>(config).unwrap();
        assert_eq!(
            parsed,
            SingleSelector::String(SingleSelectorConfig {
                default: Some("a".into()),
                choices: vec!["a".into(), "b".into(), "c".into()],
                multi: None,
            })
        )
    }

    #[test]
    fn test_single_selector_with_different_types_between_default_and_choices() {
        let config = "\
---
default: 1
choices: [a, b, c]
";
        assert!(serde_yaml::from_str::<SingleSelector>(config).is_err());
    }

    #[test]
    fn test_single_selector_with_mixed_types() {
        let config = "\
---
default: a
choices: [a, 1, c]
";
        assert!(serde_yaml::from_str::<SingleSelector>(config).is_err());
    }

    #[test]
    fn test_multi_selector() {
        let config = "\
---
default: [1]
choices: [1, 2, 3]
";
        let parsed = serde_yaml::from_str::<MultiSelector>(config).unwrap();
        assert_eq!(
            parsed,
            MultiSelector::Number(MultiSelectorConfig {
                default: Some(vec![1_f64]),
                choices: vec![1_f64, 2_f64, 3_f64],
                multi: None,
                emptyable: false,
            })
        )
    }

    #[test]
    fn test_multi_selector_with_multi_field() {
        let config = "\
---
default: [1]
choices: [1, 2, 3]
multi: true
";
        let parsed = serde_yaml::from_str::<MultiSelector>(config).unwrap();
        assert_eq!(
            parsed,
            MultiSelector::Number(MultiSelectorConfig {
                default: Some(vec![1_f64]),
                choices: vec![1_f64, 2_f64, 3_f64],
                multi: Some(LiteralTrue),
                emptyable: false,
            })
        )
    }

    #[test]
    fn test_multi_selector_for_string() {
        let config = "\
---
default: [a]
choices: [a, b, c]
";
        let parsed = serde_yaml::from_str::<MultiSelector>(config).unwrap();
        assert_eq!(
            parsed,
            MultiSelector::String(MultiSelectorConfig {
                default: Some(vec!["a".into()]),
                choices: vec!["a".into(), "b".into(), "c".into()],
                multi: None,
                emptyable: false,
            })
        )
    }

    #[test]
    fn test_multi_selector_with_different_types_between_default_and_choices() {
        let config = "\
---
default: [1]
choices: [a, b, c]
";
        assert!(serde_yaml::from_str::<MultiSelector>(config).is_err());
    }

    #[test]
    fn test_multi_selector_with_mixed_types() {
        let config = "\
---
default: [a]
choices: [a, 1, c]
";
        assert!(serde_yaml::from_str::<MultiSelector>(config).is_err());
    }

    #[test]
    fn test_prompt_kind_for_single_selector() {
        let config = "\
---
default: 1
choices: [1, 2, 3]
";
        let parsed = serde_yaml::from_str::<PromptKind>(config).unwrap();
        assert_eq!(
            parsed,
            PromptKind::SingleSelector(SingleSelector::Number(SingleSelectorConfig {
                default: Some(1_f64),
                choices: vec![1_f64, 2_f64, 3_f64],
                multi: None,
            }))
        )
    }

    #[test]
    fn test_prompt_kind_for_multi_selector() {
        let config = "\
---
default: [1]
choices: [1, 2, 3]
";
        let parsed = serde_yaml::from_str::<PromptKind>(config).unwrap();
        assert_eq!(
            parsed,
            PromptKind::MultiSelector(MultiSelector::Number(MultiSelectorConfig {
                default: Some(vec![1_f64]),
                choices: vec![1_f64, 2_f64, 3_f64],
                multi: None,
                emptyable: false
            }))
        )
    }

    #[test]
    fn test_prompt_kind_only_with_choices() {
        let config = "\
---
choices: [1, 2, 3]
";
        let parsed = serde_yaml::from_str::<PromptKind>(config).unwrap();
        assert_eq!(
            parsed,
            PromptKind::SingleSelector(SingleSelector::Number(SingleSelectorConfig {
                default: None,
                choices: vec![1_f64, 2_f64, 3_f64],
                multi: None,
            }))
        )
    }

    #[test]
    fn test_prompt_kind_with_choices_and_multi_on() {
        let config = "\
---
choices: [1, 2, 3]
multi: true
";
        let parsed = serde_yaml::from_str::<PromptKind>(config).unwrap();

        assert_eq!(
            parsed,
            PromptKind::MultiSelector(MultiSelector::Number(MultiSelectorConfig {
                default: None,
                choices: vec![1_f64, 2_f64, 3_f64],
                multi: Some(LiteralTrue),
                emptyable: false
            }))
        )
    }

    #[test]
    fn test_prompt_kind_for_confirm() {
        let config = "\
---
confirm: true
";
        let parsed = serde_yaml::from_str::<PromptKind>(config).unwrap();

        assert_eq!(
            parsed,
            PromptKind::Confirm {
                confirm: LiteralTrue,
                default: false
            }
        )
    }

    #[test]
    fn test_prompt_kind_for_confirm_with_default() {
        let config = "\
---
confirm: true
default: true
";
        let parsed = serde_yaml::from_str::<PromptKind>(config).unwrap();

        assert_eq!(
            parsed,
            PromptKind::Confirm {
                confirm: LiteralTrue,
                default: true
            }
        )
    }
}
