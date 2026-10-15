mod failure;
mod git;
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
       tightship vault sync";

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
}

#[derive(Subcommand)]
enum VaultCommand {
    Path,
    Init {
        #[arg(value_name = "folder")]
        folder: PathBuf,
    },
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
    };
    match ran {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => report(&failure),
    }
}

fn usage_problem(error: &clap::Error) -> String {
    let context = |kind| match error.get(kind) {
        Some(ContextValue::String(text)) => text.clone(),
        Some(ContextValue::Strings(texts)) => texts.join(", "),
        _ => String::new(),
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
    eprintln!("error: {failure}");
    ExitCode::FAILURE
}
