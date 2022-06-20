use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct Config {
    #[serde(default, rename(deserialize = ".meta"))]
    pub meta_config: MetaConfig,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct MetaConfig {
    #[serde(default = "default_prompt_message_for_project_name")]
    pub project_prompt_message: String,
}

fn default_prompt_message_for_project_name() -> String {
    "project name?".into()
}

impl Default for MetaConfig {
    fn default() -> Self {
        Self {
            project_prompt_message: default_prompt_message_for_project_name(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_test::{assert_de_tokens, Token};

    use super::*;

    #[test]
    fn test_deserialize_config() {
        let config = "\
---
.meta:
  project_prompt_message: what's your project name?
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
                    len: 1,
                },
                Token::Str("project_prompt_message"),
                Token::Str("what's your project name?"),
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
                    len: 1,
                },
                Token::Str("project_prompt_message"),
                Token::Str("project name?"),
                Token::StructEnd,
                Token::StructEnd,
            ],
        )
    }
}
