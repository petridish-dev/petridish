use serde::{de::Visitor, Deserialize};

#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PromptConfig {
    pub prompts: Vec<PromptItem>,
}

#[derive(Debug, PartialEq)]
pub struct PromptItem {
    name: String,
    type_: Option<PromptType>,
    kind: PromptKind,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptType {
    Number,
    Boolean,
    String,
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(untagged)]
pub enum PromptValue {
    Number(f32),
    Boolean(bool),
    String(String),
}

#[derive(Debug, PartialEq)]
pub enum PromptKind {
    Normal {
        default: Option<PromptValue>,
    },
    SingleChoice {
        choices: Vec<PromptValue>,
        default: Option<PromptValue>,
    },
    MultiChoice {
        choices: Vec<PromptValue>,
        default: Option<Vec<PromptValue>>,
    },
}

impl<'de> Deserialize<'de> for PromptItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        enum Field {
            Name,
            Type,
            Default,
            Choices,
            Multi,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter
                            .write_str("supports `name`, `type`, `default`, `choice` and `multi`")
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        match v {
                            "name" => Ok(Field::Name),
                            "type" => Ok(Field::Type),
                            "default" => Ok(Field::Default),
                            "choices" => Ok(Field::Choices),
                            "multi" => Ok(Field::Multi),
                            _ => Err(serde::de::Error::unknown_field(v, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct PromptItemVisitor;
        const FIELDS: &'static [&'static str] = &["name", "type", "default", "choices", "multi"];

        impl<'de> Visitor<'de> for PromptItemVisitor {
            type Value = PromptItem;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct PromptItem")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut name: Option<String> = None;
                let mut type_: Option<PromptType> = None;
                let mut default: Option<DefaultPrompt> = None;
                let mut choices: Option<Vec<PromptValue>> = None;
                let mut multi: Option<bool> = None;

                #[derive(Deserialize, Debug)]
                #[serde(untagged)]
                enum DefaultPrompt {
                    Single(PromptValue),
                    Multi(Vec<PromptValue>),
                }

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        Field::Type => {
                            if type_.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            type_ = Some(map.next_value()?);
                        }
                        Field::Default => {
                            if default.is_some() {
                                return Err(serde::de::Error::duplicate_field("default"));
                            }
                            default = Some(map.next_value()?);
                        }
                        Field::Choices => {
                            if choices.is_some() {
                                return Err(serde::de::Error::duplicate_field("choices"));
                            }
                            choices = Some(map.next_value()?);
                        }
                        Field::Multi => {
                            if multi.is_some() {
                                return Err(serde::de::Error::duplicate_field("multi"));
                            }
                            multi = Some(map.next_value()?);
                        }
                    }
                }

                let name = name.ok_or_else(|| serde::de::Error::missing_field("name"))?;
                let mut single_default: Option<PromptValue> = None;
                let mut multi_default: Option<Vec<PromptValue>> = None;
                match default {
                    Some(DefaultPrompt::Multi(v)) => multi_default = Some(v),
                    Some(DefaultPrompt::Single(v)) => single_default = Some(v),
                    _ => (),
                }

                if let Some(choices) = choices {
                    match multi {
                        Some(true) => Ok(PromptItem {
                            name,
                            type_,
                            kind: PromptKind::MultiChoice {
                                default: multi_default,
                                choices,
                            },
                        }),
                        _ => Ok(PromptItem {
                            name,
                            type_,
                            kind: PromptKind::SingleChoice {
                                default: single_default,
                                choices,
                            },
                        }),
                    }
                } else {
                    Ok(PromptItem {
                        name,
                        type_,
                        kind: PromptKind::Normal {
                            default: single_default,
                        },
                    })
                }
            }
        }

        deserializer.deserialize_struct("PromptItem", FIELDS, PromptItemVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml;

    #[test]
    fn it_deserialize_prompt_item() {
        let yaml = r#"
name: hello
default: [1]
choices: [2, 1]
multi: true
"#;
        let result = serde_yaml::from_str::<PromptItem>(yaml);
        println!("{:?}", result);
    }
}
