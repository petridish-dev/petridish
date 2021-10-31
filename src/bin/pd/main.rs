use std::path::PathBuf;
use structopt::clap::AppSettings::{ColorAuto, ColoredHelp};
use structopt::StructOpt;

use clap::{crate_authors, crate_description};
use miette::Result;
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
    println!("{:#?}", app);

    Ok(())
}
