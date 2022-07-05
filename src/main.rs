use std::{
    collections::HashMap,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use clap::{builder::ArgAction, Parser, Subcommand};
use crossterm::style::{Color, Stylize};
use inquire::error::InquireError;
use petridish::{
    cache::Cache,
    config::{Config, Prompt},
    error::Error,
    render::Render,
    try_new_repo,
};
use tabled::{
    object::{Columns, FirstRow, Segment},
    Alignment, Format, Modify, Style, Table, Tabled,
};
use tera::Context;
use termimad::*;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[clap(about = "Generate new project")]
    New {
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
            long = "skip-if-exists",
            action,
            help = "Skip the file if it already exists"
        )]
        skip: bool,

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
    },
    #[clap(about = "List all cached templates")]
    List,
}

fn entry() -> petridish::error::Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::New {
            template_uri,
            force,
            skip,
            output_dir,
            extra_context: _,
            auth,
            branch,
        } => {
            let mut context = HashMap::new();
            if let Some(auth) = auth.as_ref() {
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

            if let Some(branch) = branch.as_ref() {
                context.insert("branch".to_string(), branch.to_string());
            }

            let repo = if regex::Regex::new(r"^[\w-]+$")
                .unwrap()
                .is_match(&template_uri)
                && !Path::new(&template_uri).exists()
            {
                let path = Cache::get(&template_uri)
                    .ok_or_else(|| Error::RepoNotFoundInCache(template_uri.to_string()))?;
                try_new_repo(path.display().to_string(), context.clone())?
            } else {
                let repo = try_new_repo(template_uri.clone(), context.clone())?;
                if repo.need_cache()
                    && Cache::get(repo.name()).is_some()
                    && !inquire::Confirm::new(&format!(
                        "You've downloaded '{}' before. Is it okay to re-download it?",
                        repo.name()
                    ))
                    .with_default(true)
                    .prompt()?
                {
                    return Ok(());
                }

                match repo.download() {
                    Err(Error::GitError(e)) => {
                        if e.code() == git2::ErrorCode::Auth {
                            let username = inquire::Text::new("git username").prompt()?;
                            let password = inquire::Password::new("git password").prompt()?;
                            context.insert("username".to_string(), username);
                            context.insert("password".to_string(), password);
                            let repo = try_new_repo(template_uri, context)?;
                            repo.download()?;
                            repo
                        } else {
                            return Err(Error::GitError(e));
                        }
                    }
                    Err(e) => return Err(e),
                    _ => repo,
                }
            };

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

            // show description
            let description = petridish_config
                .petridish_config
                .long_description
                .or(petridish_config.petridish_config.short_description);
            if let Some(description) = description {
                let mut skin = MadSkin::default();
                skin.set_headers_fg(rgb(255, 187, 0));
                skin.bold.set_fg(Color::Yellow);
                skin.italic.set_fgbg(Color::Magenta, rgb(30, 30, 40));
                println!("{}", skin.term_text(&description));
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

            let output_path = output_dir.unwrap_or_default();
            let render = Render::new(
                repo.repo_dir(),
                &entry_dir_name,
                output_path,
                prompt_context,
                force,
                skip,
            );
            render.render()?;
        }
        Commands::List => {
            let mut templates = vec![];

            for path in Cache::list() {
                let name = path.file_name().unwrap().to_str().unwrap();
                let config = path.join("petridish.toml");
                if !config.exists() {
                    continue;
                }
                let config = toml::from_str::<Config>(&read_to_string(&config).unwrap()).unwrap();
                let description = config
                    .petridish_config
                    .short_description
                    .unwrap_or_default();
                templates.push(CachedTemplate {
                    name: name.to_string(),
                    description,
                });
            }
            println!(
                "{}",
                Table::new(templates)
                    .with(Style::blank())
                    .with(Modify::new(Segment::all()).with(Alignment::left()))
                    .with(Modify::new(FirstRow).with(Format::new(|s| s.yellow().to_string())))
                    .with(
                        Modify::new(Columns::single(1)).with(Format::new(|s| s.blue().to_string()))
                    )
            );
        }
    }

    Ok(())
}

#[derive(Tabled)]
struct CachedTemplate {
    name: String,
    description: String,
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
