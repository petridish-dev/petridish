use std::env;
use std::fmt::Debug;
use std::path::PathBuf;

use clap::{crate_authors, crate_description};
use dialoguer::{theme::ColorfulTheme, Input, Select};
use miette::Result;
use petridish::config::PromptConfig;
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
    for prompt in config.prompts {
        let input: String = {
            if let Some(choices) = prompt.choices {
                let default: usize = match prompt.default {
                    Some(default) => *(&choices.iter().position(|i| i == &default).unwrap()),
                    None => 0,
                };

                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt(prompt.message.unwrap_or(prompt.name.clone()))
                    .default(default)
                    .items(&choices)
                    .interact()
                    .unwrap();
                choices[selection].clone()
            } else {
                if let Some(default) = prompt.default {
                    let default = tera.render_str(&default, &context).unwrap();
                    Input::with_theme(&ColorfulTheme::default())
                        .with_prompt(prompt.message.unwrap_or(prompt.name.clone()))
                        .default(default)
                        .interact_text()
                        .unwrap()
                } else {
                    Input::with_theme(&ColorfulTheme::default())
                        .with_prompt(prompt.message.unwrap_or(prompt.name.clone()))
                        .interact_text()
                        .unwrap()
                }
            }
        };

        context.insert(prompt.name, &input);
    }

    println!("{:?}", context);
    Ok(())
}
