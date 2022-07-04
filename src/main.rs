use std::fs::read_to_string;
use std::{collections::HashMap, path::PathBuf};

use clap::{builder::ArgAction, Parser};
use inquire::error::InquireError;
use petridish::{
    config::{Config, Prompt},
    error::Error,
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
        long = "overwrite-if-exists",
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

fn entry() -> petridish::error::Result<()> {
    let args = Args::parse();
    let mut context = HashMap::new();
    if let Some(auth) = args.auth.as_ref() {
        let splitted_auth = auth.split(':').collect::<Vec<&str>>();
        if splitted_auth.len() != 2 {
            Err(Error::ArgsError(format!(
                "auth '{}' is invalid, should be like <username>:<password>",
                auth
            )))?;
        }
        context.insert("username".to_string(), splitted_auth[0].to_string());
        context.insert("password".to_string(), splitted_auth[1].to_string());
    }

    if let Some(branch) = args.branch.as_ref() {
        context.insert("branch".to_string(), branch.to_string());
    }

    let repo = try_new_repo(args.template_uri, context)?;
    let petridish_config = repo.repo_dir().join("petridish.toml");
    let petridish_config =
        toml::from_str::<Config>(&read_to_string(&petridish_config).map_err(|e| {
            Error::PathNotFound {
                source: e,
                path: petridish_config,
            }
        })?)?;
    let entry_dir_name = format!(
        "{{{{ {} }}}}",
        petridish_config.petridish_config.project_var_name
    );
    let entry_dir = repo.repo_dir().join(&entry_dir_name);
    if !entry_dir.exists() {
        return Err(Error::PathNotFound {
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No such file or directory (os error 2)",
            ),
            path: entry_dir,
        });
    }

    // start prompting
    let mut prompt_context = Context::new();

    let project_name =
        inquire::Text::new(&petridish_config.petridish_config.project_prompt).prompt()?;

    prompt_context.insert(
        petridish_config.petridish_config.project_var_name,
        &project_name,
    );
    for prompt_type in petridish_config.prompts {
        prompt_type.prompt(&mut prompt_context)?;
    }

    let output_path = args.output_dir.unwrap_or_default();
    let render = Render::new(
        repo.repo_dir(),
        &entry_dir_name,
        output_path,
        prompt_context,
        args.force,
    );
    render.render()?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    if let Err(e) = entry() {
        if matches!(
            e,
            Error::PromptError(InquireError::OperationCanceled)
                | Error::PromptError(InquireError::OperationInterrupted)
        ) {
            return Ok(());
        }

        return Err(e)?;
    }

    Ok(())
}
