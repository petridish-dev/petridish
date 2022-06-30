use std::fs::read_to_string;
use std::{collections::HashMap, path::PathBuf};

use clap::{builder::ArgAction, Parser};
use inquire::validator::Validation;
use petridish::{
    config::{BoolPrompt, Config},
    error::{Error, Result},
    render::Render,
    try_new_repo,
};
use tera::Context;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(value_parser, help = "The petridish template uri or local path")]
    template_uri: String,

    #[clap(
        short,
        long = "overwrite_if_exists",
        action,
        help = "Overwrite the contents of the output directory if it already exists"
    )]
    force: bool,

    #[clap(
        short,
        long,
        value_parser,
        help = "Where to output the generated project dir into"
    )]
    output_dir: Option<PathBuf>,

    #[clap(
        value_parser,
        action = ArgAction::Set,
        default_value = "",
        hide_default_value = true,
        help = "Add default prompt values, format should be like <key>=<value>"
    )]
    extra_context: Vec<String>,

    #[clap(
        value_parser,
        long,
        help = "The username and password used for authorization, format should be like <username>:<password>"
    )]
    auth: Option<String>,

    #[clap(
        value_parser,
        long,
        help = "Check into the branch, tag or commit after git clone"
    )]
    branch: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut context = HashMap::new();
    if let Some(auth) = args.auth.as_ref() {
        let splitted_auth = auth.split(':').collect::<Vec<&str>>();
        if splitted_auth.len() != 2 {
            return Err(Error::ArgsError(format!(
                "auth '{}' is invalid, should be like <username>:<password>",
                auth
            )));
        }
        context.insert("username".to_string(), splitted_auth[0].to_string());
        context.insert("password".to_string(), splitted_auth[1].to_string());
    }

    if let Some(branch) = args.branch.as_ref() {
        context.insert("branch".to_string(), branch.to_string());
    }

    let repo = try_new_repo(args.template_uri, context)?;
    let petridish_config = repo.repo_dir().join("petridish.toml");
    if !petridish_config.exists() {
        return Err(Error::ConfigNotFound(format!(
            "{} not exists",
            petridish_config.display()
        )));
    }

    let petridish_config =
        toml::from_str::<Config>(&read_to_string(petridish_config).unwrap()).unwrap();
    let entry_dir_name = format!(
        "{{{{ {} }}}}",
        petridish_config.petridish_config.project_var_name
    );
    let entry_dir = repo.repo_dir().join(&entry_dir_name);
    if !entry_dir.exists() {
        return Err(Error::InvalidRepo(format!(
            "not found entry dir '{}'",
            entry_dir.display()
        )));
    }

    // start prompting
    let mut prompt_context = Context::new();

    let project_name = inquire::Text::new(&petridish_config.petridish_config.project_prompt)
        .prompt()
        .unwrap();
    prompt_context.insert(
        petridish_config.petridish_config.project_var_name,
        &project_name,
    );
    for prompt_config in petridish_config.prompts {
        let prompt_msg = prompt_config
            .prompt
            .unwrap_or_else(|| prompt_config.name.clone());
        match prompt_config.prompt_type {
            petridish::config::PromptType::String(v) => match v {
                petridish::config::StringPrompt::MultiSelector {
                    multi: _,
                    choices,
                    default,
                } => {
                    let defaults = {
                        match default {
                            Some(default) => choices
                                .iter()
                                .enumerate()
                                .filter(|(_, choice)| default.contains(choice))
                                .map(|(idx, _)| idx)
                                .collect(),
                            None => vec![],
                        }
                    };

                    let selections = inquire::MultiSelect::new(&prompt_msg, choices)
                        .with_default(&defaults)
                        .prompt()
                        .unwrap();

                    prompt_context.insert(prompt_config.name, &selections);
                }
                petridish::config::StringPrompt::SingleSelector { choices, default } => {
                    let default: usize = match default {
                        Some(default) => choices.iter().position(|i| i == &default).unwrap(),
                        None => 0,
                    };
                    let value = inquire::Select::new(&prompt_msg, choices)
                        .with_starting_cursor(default)
                        .prompt()
                        .unwrap();
                    prompt_context.insert(prompt_config.name, &value);
                }
                petridish::config::StringPrompt::Normal { default, regex } => {
                    let default = default.unwrap_or_default();
                    let regex = regex::Regex::new(&regex.unwrap_or_else(|| ".*".into())).unwrap();
                    let value = inquire::Text::new(&prompt_msg)
                        .with_default(&default)
                        .with_validator(&|v| {
                            if regex.is_match(v) {
                                Ok(Validation::Valid)
                            } else {
                                Ok(Validation::Invalid(
                                    format!("'not match regex '{}'", regex).into(),
                                ))
                            }
                        })
                        .prompt()
                        .unwrap();
                    prompt_context.insert(prompt_config.name, &value);
                }
            },
            petridish::config::PromptType::Number(v) => match v {
                petridish::config::NumberPrompt::MultiSelector {
                    multi: _,
                    choices,
                    default,
                } => {
                    let defaults = {
                        match default {
                            Some(default) => choices
                                .iter()
                                .enumerate()
                                .filter(|(_, choice)| default.contains(choice))
                                .map(|(idx, _)| idx)
                                .collect(),
                            None => vec![],
                        }
                    };

                    let selections = inquire::MultiSelect::new(&prompt_msg, choices)
                        .with_default(&defaults)
                        .prompt()
                        .unwrap();

                    prompt_context.insert(prompt_config.name, &selections);
                }
                petridish::config::NumberPrompt::SingleSelector { choices, default } => {
                    let default: usize = match default {
                        Some(default) => choices.iter().position(|i| i == &default).unwrap(),
                        None => 0,
                    };
                    let value = inquire::Select::new(&prompt_msg, choices)
                        .with_starting_cursor(default)
                        .prompt()
                        .unwrap();
                    prompt_context.insert(prompt_config.name, &value);
                }
                petridish::config::NumberPrompt::Normal { default, min, max } => {
                    let default = default.unwrap_or_default();
                    let value = match (min, max) {
                        (Some(min), Some(max)) => inquire::CustomType::<f64>::new(&prompt_msg)
                            .with_default((default, &|v| v.to_string()))
                            .with_error_message("Please type a valid number")
                            .with_help_message(&format!("range: {} <= value <= {}", min, max))
                            .with_parser(&|v| {
                                let v = v.parse::<f64>().map_err(|_| ())?;
                                if v < min || v > max {
                                    Err(())
                                } else {
                                    Ok(v)
                                }
                            })
                            .prompt()
                            .unwrap(),
                        (Some(min), None) => inquire::CustomType::<f64>::new(&prompt_msg)
                            .with_default((default, &|v| v.to_string()))
                            .with_error_message("Please type a valid number")
                            .with_help_message(&format!("range: {} <= value", min))
                            .with_parser(&|v| {
                                let v = v.parse::<f64>().map_err(|_| ())?;
                                if v < min {
                                    Err(())
                                } else {
                                    Ok(v)
                                }
                            })
                            .prompt()
                            .unwrap(),
                        (None, Some(max)) => inquire::CustomType::<f64>::new(&prompt_msg)
                            .with_default((default, &|v| v.to_string()))
                            .with_error_message("Please type a valid number")
                            .with_help_message(&format!("range: value <= {}", max))
                            .with_parser(&|v| {
                                let v = v.parse::<f64>().map_err(|_| ())?;
                                if v > max {
                                    Err(())
                                } else {
                                    Ok(v)
                                }
                            })
                            .prompt()
                            .unwrap(),
                        _ => inquire::CustomType::<f64>::new(&prompt_msg)
                            .with_default((default, &|v| v.to_string()))
                            .with_error_message("Please type a valid number")
                            .prompt()
                            .unwrap(),
                    };

                    prompt_context.insert(prompt_config.name, &value);
                }
            },
            petridish::config::PromptType::Bool(BoolPrompt { default }) => {
                let value = inquire::Confirm::new(&prompt_msg)
                    .with_default(default)
                    .prompt()
                    .unwrap();
                prompt_context.insert(prompt_config.name, &value);
            }
        }
    }

    let output_path = args.output_dir.unwrap_or_default();
    let render = Render::try_new(
        repo.repo_dir(),
        &entry_dir_name,
        output_path,
        prompt_context,
    )?;
    render.render()?;

    Ok(())
}
