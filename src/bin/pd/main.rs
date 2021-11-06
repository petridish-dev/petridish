use std::env;
use std::fmt::Debug;
use std::path::PathBuf;

use clap::{crate_authors, crate_description};
use dialoguer::{theme::ColorfulTheme, Input, Select};
use dialoguer::{Confirm, MultiSelect as DialoguerMultiSelect};
use miette::Result;
use petridish::config::{
    MultiSelect, MultiSelectType, PromptConfig, SingleSelect, SingleSelectType, Value,
};
use petridish::render::Render;
use petridish::source::new_source;
use structopt::clap::AppSettings::{ColorAuto, ColoredHelp};
use structopt::StructOpt;
use tera::{Context, Tera};

#[derive(Debug, StructOpt)]
#[structopt(name="petridish", author = crate_authors!(), about = crate_description!(), setting(ColorAuto), setting(ColoredHelp))]
struct App {
    #[structopt(
        short = "o",
        long = "output",
        default_value = ".",
        help = "Where to output the generated project",
        parse(from_os_str)
    )]
    output: PathBuf,

    #[structopt(
        short = "f",
        long = "force",
        help = "Overwrite the generate project if it already exists"
    )]
    force: bool,

    #[structopt(name = "TEMPLATE", help = "The petridish template")]
    template: String,
}

fn main() -> Result<()> {
    let app = App::from_args();
    let source = new_source(&app.template)?;
    let config_path = source.get_config()?;
    let config = PromptConfig::from_yaml_path(&config_path)?;
    let mut context = Context::new();
    let mut tera = Tera::default();

    let entry_dir: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(config.entry_dir_prompt_message)
        .interact_text()
        .unwrap();
    let entry_dir_var_name = config
        .entry_dir
        .strip_prefix("{{")
        .unwrap()
        .strip_suffix("}}")
        .unwrap()
        .trim();
    context.insert(entry_dir_var_name, &entry_dir);

    for prompt in config.prompts {
        match prompt.kind {
            petridish::config::PromptKind::Default { default } => match default {
                Some(Value::String(default)) => {
                    let default = tera.render_str(&default, &context).unwrap();
                    let value = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt(prompt.message.unwrap_or_else(|| prompt.name.clone()))
                        .default(default)
                        .interact_text()
                        .unwrap();
                    context.insert(prompt.name, &value);
                }
                Some(Value::Number(default)) => {
                    let default = default.as_f64().unwrap();
                    let value = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt(prompt.message.unwrap_or_else(|| prompt.name.clone()))
                        .default(default)
                        .interact_text()
                        .unwrap();
                    context.insert(prompt.name, &value);
                }
                _ => {
                    let value: String = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt(prompt.message.unwrap_or_else(|| prompt.name.clone()))
                        .interact_text()
                        .unwrap();
                    context.insert(prompt.name, &value);
                }
            },
            petridish::config::PromptKind::Confirm {
                confirm: _,
                default,
            } => {
                let confirm = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt(prompt.message.unwrap_or_else(|| prompt.name.clone()))
                    .default(default)
                    .show_default(true)
                    .wait_for_newline(true)
                    .interact()
                    .unwrap();
                context.insert(prompt.name, &confirm);
            }
            petridish::config::PromptKind::SingleSelect(value_type) => match value_type {
                SingleSelectType::String(SingleSelect {
                    default,
                    choices,
                    multi: _,
                }) => {
                    let default: usize = match default {
                        Some(default) => choices.iter().position(|i| i == &default).unwrap(),
                        None => 0,
                    };
                    let selection = Select::with_theme(&ColorfulTheme::default())
                        .with_prompt(prompt.message.unwrap_or_else(|| prompt.name.clone()))
                        .default(default)
                        .items(&choices)
                        .interact()
                        .unwrap();
                    let value = &choices[selection];
                    context.insert(prompt.name, value);
                }
                SingleSelectType::Number(SingleSelect {
                    default,
                    choices,
                    multi: _,
                }) => {
                    let default: usize = match default {
                        Some(default) => choices.iter().position(|i| i == &default).unwrap(),
                        None => 0,
                    };
                    let selection = Select::with_theme(&ColorfulTheme::default())
                        .with_prompt(prompt.message.unwrap_or_else(|| prompt.name.clone()))
                        .default(default)
                        .items(&choices)
                        .interact()
                        .unwrap();
                    let value = &choices[selection];
                    context.insert(prompt.name, &value.as_f64().unwrap());
                }
            },
            petridish::config::PromptKind::MultiSelect(value_type) => match value_type {
                MultiSelectType::String(MultiSelect {
                    default,
                    choices,
                    multi: _,
                    emptyable,
                }) => {
                    let defaults = {
                        match default {
                            Some(defaults) => choices
                                .iter()
                                .map(|choice| defaults.contains(choice))
                                .collect(),
                            None => vec![false; choices.len()],
                        }
                    };

                    let prompt_message = prompt.message.unwrap_or_else(|| prompt.name.clone());

                    let selections = loop {
                        let selections =
                            DialoguerMultiSelect::with_theme(&ColorfulTheme::default())
                                .with_prompt(&prompt_message)
                                .items(&choices[..])
                                .defaults(&defaults[..])
                                .interact()
                                .unwrap();
                        if !emptyable && selections.is_empty() {
                            println!("You did not select anything :(");
                            continue;
                        }
                        break selections;
                    };
                    let selections = selections
                        .iter()
                        .map(|idx| choices[*idx].clone())
                        .collect::<Vec<_>>();
                    context.insert(prompt.name, &selections);
                }
                MultiSelectType::Number(MultiSelect {
                    default,
                    choices,
                    multi: _,
                    emptyable,
                }) => {
                    let defaults = {
                        match default {
                            Some(defaults) => choices
                                .iter()
                                .map(|choice| defaults.contains(choice))
                                .collect(),
                            None => vec![false; choices.len()],
                        }
                    };

                    let prompt_message = prompt.message.unwrap_or_else(|| prompt.name.clone());

                    let selections = loop {
                        let selections =
                            DialoguerMultiSelect::with_theme(&ColorfulTheme::default())
                                .with_prompt(&prompt_message)
                                .items(&choices[..])
                                .defaults(&defaults[..])
                                .interact()
                                .unwrap();
                        if !emptyable && selections.is_empty() {
                            println!("You did not select anything :(");
                            continue;
                        }
                        break selections;
                    };
                    let selections = selections
                        .iter()
                        .map(|idx| choices[*idx].as_f64().unwrap())
                        .collect::<Vec<_>>();
                    context.insert(prompt.name, &selections);
                }
            },
        }
    }

    let render = Render::try_new(
        source.get_template(),
        &config.entry_dir,
        &app.output,
        context,
    )?;
    render.render()?;
    Ok(())
}
