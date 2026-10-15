mod failure;
mod git;
mod hook;
mod project;
mod vault;

use std::panic;
use std::path::PathBuf;
use std::process::{self, ExitCode};

use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{Parser, Subcommand};

use failure::Failure;

pub const USAGE: &str = "\
Usage: tightship vault path
       tightship vault init <folder>
       tightship vault search [--everywhere] <term>...
       tightship vault new --scope <project|all> --type <preference|decision|gotcha|project>
                           --title <title> --why <line> --source <user|session|repo>
                           [--tags <a,b>] [--pin] [--created <YYYY-MM-DD>] < body
       tightship vault lint
       tightship vault sync
       tightship vault handoff release --title <title> < body
       tightship vault handoff claim";

#[derive(Parser)]
#[command(name = "tightship")]
struct Arguments {
    #[command(subcommand)]
    area: Area,
}

#[derive(Subcommand)]
enum Area {
    #[command(subcommand)]
    Vault(VaultCommand),
    #[command(subcommand)]
    Hook(HookCommand),
}

#[derive(Subcommand)]
enum HookCommand {
    Start,
    Export,
    Checkpoint,
    Prompt,
}

#[derive(Subcommand)]
enum VaultCommand {
    Path,
    Init {
        #[arg(value_name = "folder")]
        folder: PathBuf,
    },
    Search {
        #[arg(long)]
        everywhere: bool,
        #[arg(value_name = "term", required = true)]
        terms: Vec<String>,
    },
    New(vault::new::Arguments),
    Lint,
    Sync,
    #[command(subcommand)]
    Handoff(vault::handoff::Command),
}

fn main() -> ExitCode {
    panic::set_hook(Box::new(|info| {
        let message = info.payload_as_str().unwrap_or("no message");
        eprintln!("error: tightship stopped on an internal fault: {message}");
        process::exit(1);
    }));
    let arguments = match Arguments::try_parse() {
        Ok(arguments) => arguments,
        Err(error) if error.kind() == ErrorKind::DisplayHelp => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Err(error) => return report(&Failure::Usage(usage_problem(&error))),
    };
    let ran = match arguments.area {
        Area::Vault(VaultCommand::Path) => vault::path::run(),
        Area::Vault(VaultCommand::Init { folder }) => vault::init::run(&folder),
        Area::Vault(VaultCommand::Search { everywhere, terms }) => {
            vault::search::run(&terms, everywhere)
        }
        Area::Vault(VaultCommand::New(arguments)) => vault::new::run(&arguments),
        Area::Vault(VaultCommand::Lint) => vault::lint::run(),
        Area::Vault(VaultCommand::Sync) => vault::sync::run(),
        Area::Vault(VaultCommand::Handoff(command)) => vault::handoff::run(&command),
        Area::Hook(HookCommand::Start) => hook::start::run(),
        Area::Hook(HookCommand::Export) => hook::export::run(),
        Area::Hook(HookCommand::Checkpoint) => hook::checkpoint::run(),
        Area::Hook(HookCommand::Prompt) => hook::prompt::run(),
    };
    match ran {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => report(&failure),
    }
}

fn usage_problem(error: &clap::Error) -> String {
    let context = |kind| {
        let texts = match error.get(kind) {
            Some(ContextValue::String(text)) => vec![text.as_str()],
            Some(ContextValue::Strings(texts)) => texts.iter().map(String::as_str).collect(),
            _ => Vec::new(),
        };
        let names: Vec<&str> = texts
            .iter()
            .filter_map(|text| text.split_whitespace().next())
            .collect();
        names.join(", ")
    };
    match error.kind() {
        ErrorKind::InvalidSubcommand => {
            format!(
                "unknown subcommand {}",
                context(ContextKind::InvalidSubcommand)
            )
        }
        ErrorKind::UnknownArgument => {
            format!("unknown argument {}", context(ContextKind::InvalidArg))
        }
        ErrorKind::InvalidValue => {
            format!("{} needs a value", context(ContextKind::InvalidArg))
        }
        ErrorKind::MissingRequiredArgument => {
            format!("missing {}", context(ContextKind::InvalidArg))
        }
        ErrorKind::MissingSubcommand | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            "missing subcommand".to_owned()
        }
        kind => kind.to_string(),
    }
}

fn report(failure: &Failure) -> ExitCode {
    if !matches!(failure, Failure::LintFailed) {
        eprintln!("error: {failure}");
    }
    ExitCode::FAILURE
}
