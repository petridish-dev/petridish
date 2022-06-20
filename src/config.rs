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
            project_prompt_message: default_prompt_message_for_project_name(),
            project_var_name: default_project_var_name(),
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
  project_var_name: project
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
                    len: 2,
                },
                Token::Str("project_prompt_message"),
                Token::Str("what's your project name?"),
                Token::Str("project_var_name"),
                Token::Str("project"),
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
                    len: 2,
                },
                Token::Str("project_prompt_message"),
                Token::Str("project name?"),
                Token::Str("project_var_name"),
                Token::Str("project_name"),
                Token::StructEnd,
                Token::StructEnd,
            ],
        )
    }
}
