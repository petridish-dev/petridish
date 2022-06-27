use std::fs::read_to_string;
use std::{collections::HashMap, path::PathBuf};

use clap::{builder::ArgAction, Parser};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use petridish::{
    config::Config,
    error::{Error, Result},
    prompt::BoolPrompt,
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

    let project_name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(petridish_config.petridish_config.project_prompt)
        .interact_text()
        .unwrap();
    prompt_context.insert(
        petridish_config.petridish_config.project_var_name,
        &project_name,
    );
    for (prompt_name, prompt_config) in petridish_config.prompts {
        let prompt_msg = prompt_config.prompt.unwrap_or_else(|| prompt_name.clone());
        match prompt_config.kind {
            petridish::prompt::PromptKind::String(v) => match v {
                petridish::prompt::StringPrompt::MultiSelector {
                    multi: _,
                    choices,
                    default,
                } => {
                    let defaults = {
                        match default {
                            Some(default) => choices
                                .iter()
                                .map(|choice| default.contains(choice))
                                .collect(),
                            None => vec![false; choices.len()],
                        }
                    };
                    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
                        .with_prompt(&prompt_msg)
                        .items(&choices[..])
                        .defaults(&defaults[..])
                        .interact()
                        .unwrap();
                    let selections = selections
                        .iter()
                        .map(|idx| choices[*idx].clone())
                        .collect::<Vec<_>>();
                    prompt_context.insert(prompt_name, &selections);
                }
                petridish::prompt::StringPrompt::SingleSelector { choices, default } => {
                    let default: usize = match default {
                        Some(default) => choices.iter().position(|i| i == &default).unwrap(),
                        None => 0,
                    };
                    let selection = Select::with_theme(&ColorfulTheme::default())
                        .with_prompt(&prompt_msg)
                        .items(&choices)
                        .default(default)
                        .interact()
                        .unwrap();
                    let value = &choices[selection];
                    prompt_context.insert(prompt_name, value);
                }
                petridish::prompt::StringPrompt::Normal { default, regex } => {
                    let default = default.unwrap_or_default();
                    let regex = regex.map(|pattern| regex::Regex::new(&pattern).unwrap());
                    let value = loop {
                        let value = Input::with_theme(&ColorfulTheme::default())
                            .with_prompt(&prompt_msg)
                            .default(default.clone())
                            .interact_text()
                            .unwrap();
                        if let Some(regex) = &regex {
                            if !regex.is_match(&value) {
                                println!("not match regex: {}", regex);
                                continue;
                            }
                        }
                        break value;
                    };
                    prompt_context.insert(prompt_name, &value);
                }
            },
            petridish::prompt::PromptKind::Number(v) => match v {
                petridish::prompt::NumberPrompt::MultiSelector {
                    multi: _,
                    choices,
                    default,
                } => {
                    let defaults = {
                        match default {
                            Some(default) => choices
                                .iter()
                                .map(|choice| default.contains(choice))
                                .collect(),
                            None => vec![false; choices.len()],
                        }
                    };
                    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
                        .with_prompt(&prompt_msg)
                        .items(&choices[..])
                        .defaults(&defaults[..])
                        .interact()
                        .unwrap();
                    let selections = selections
                        .iter()
                        .map(|idx| choices[*idx])
                        .collect::<Vec<_>>();
                    prompt_context.insert(prompt_name, &selections);
                }
                petridish::prompt::NumberPrompt::SingleSelector { choices, default } => {
                    let default: usize = match default {
                        Some(default) => choices.iter().position(|i| i == &default).unwrap(),
                        None => 0,
                    };
                    let selection = Select::with_theme(&ColorfulTheme::default())
                        .with_prompt(&prompt_msg)
                        .items(&choices)
                        .default(default)
                        .interact()
                        .unwrap();
                    let value = &choices[selection];
                    prompt_context.insert(prompt_name, value);
                }
                petridish::prompt::NumberPrompt::Normal { default, min, max } => {
                    let default = default.unwrap_or_default();
                    let value = loop {
                        let value = Input::with_theme(&ColorfulTheme::default())
                            .with_prompt(&prompt_msg)
                            .default(default)
                            .interact_text()
                            .unwrap();
                        if let Some(min) = min {
                            if value < min {
                                println!("{} is less than {}", value, min);
                                continue;
                            }
                        }
                        if let Some(max) = max {
                            if value > max {
                                println!("{} is greater than {}", value, max);
                                continue;
                            }
                        }
                        break value;
                    };
                    prompt_context.insert(prompt_name, &value);
                }
            },
            petridish::prompt::PromptKind::Bool(BoolPrompt { default }) => {
                let value = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt(&prompt_msg)
                    .default(default)
                    .wait_for_newline(true)
                    .interact()
                    .unwrap();
                prompt_context.insert(prompt_name, &value);
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
