//! The `tightship` entry point: parses the command line, runs the command, and reports a failure.

mod failure;
mod hook;
mod options;
mod project;
mod status;
mod system;
mod update;
mod vault;

use std::panic;
use std::path::PathBuf;
use std::process::{self, ExitCode};

use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{Parser, Subcommand};

use failure::Failure;

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
    #[command(subcommand)]
    StatusLine(StatusLineCommand),
    #[command(subcommand)]
    Options(OptionsCommand),
    Update,
}

#[derive(Subcommand)]
enum HookCommand {
    Start,
    Export,
    Checkpoint,
    StatusLine,
    ContractPart { file: String },
    Map,
    Clean,
    Update,
    MarketplaceAutoUpdate,
}

#[derive(Subcommand)]
enum StatusLineCommand {
    Render,
}

#[derive(Subcommand)]
enum OptionsCommand {
    Set { name: String, value: String },
    List,
    Reset,
    Disabled { name: String },
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
            println!("{}", failure::USAGE);
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
        Area::Hook(HookCommand::StatusLine) => hook::status_line::run(),
        Area::Hook(HookCommand::ContractPart { file }) => hook::contract_part::run(&file),
        Area::Hook(HookCommand::Map) => hook::map::run(),
        Area::Hook(HookCommand::Clean) => hook::clean::run(),
        Area::Hook(HookCommand::Update) => hook::update::run(),
        Area::Hook(HookCommand::MarketplaceAutoUpdate) => hook::marketplace_auto_update::run(),
        Area::Update => update::run(),
        Area::StatusLine(StatusLineCommand::Render) => status::render::run(),
        Area::Options(OptionsCommand::Set { name, value }) => options::set(&name, &value),
        Area::Options(OptionsCommand::List) => options::list(),
        Area::Options(OptionsCommand::Reset) => options::reset(),
        Area::Options(OptionsCommand::Disabled { name }) => match options::disabled(&name) {
            Ok(true) => Ok(()),
            Ok(false) => return ExitCode::from(1),
            Err(failure) => Err(failure),
        },
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
