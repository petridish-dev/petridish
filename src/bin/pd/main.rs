use std::path::PathBuf;
use structopt::clap::AppSettings::{ColorAuto, ColoredHelp};
use structopt::StructOpt;

use clap::{crate_authors, crate_description};
use miette::Result;
use petridish::config::PromptConfig;
use petridish::source::new_source;
use std::env;
use std::fmt::Debug;

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
    println!("{:?}", config);
    Ok(())
}
