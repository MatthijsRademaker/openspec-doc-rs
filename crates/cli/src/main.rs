//! The `openspec-doc` binary: argument parsing, shared project-root
//! resolution, and subcommand dispatch.

mod cli;
mod comment;
mod error;
mod hook;
mod root;
mod scratch;
mod serve;
mod summary;

use std::process::ExitCode;

use clap::Parser;

use crate::cli::{Cli, Command, CommentCommand, HookCommand, ScratchCommand};
use crate::error::Error;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprint_chain(&error);
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Error> {
    let cli = Cli::parse();
    let project = root::resolve(cli.root.as_deref())?;

    match cli.command {
        Command::Summary => summary::run(project),
        Command::Serve {
            host,
            port,
            no_open,
        } => serve::run(project, host, port, no_open),
        Command::Hook { command } => match command {
            HookCommand::Stop { agent } => hook::stop(project, agent),
            HookCommand::Explore { agent } => hook::explore(project, agent),
        },
        Command::Comment { command } => match command {
            CommentCommand::Add {
                scope,
                artifact,
                selected_text,
                body,
            } => comment::add(project, scope.key(), &artifact, &selected_text, &body),
            CommentCommand::List { scope } => comment::list(project, scope.key()),
            CommentCommand::Reply {
                scope,
                comment,
                body,
            } => comment::reply(project, scope.key(), &comment, &body),
            CommentCommand::Resolve { scope, comment } => {
                comment::resolve(project, scope.key(), &comment)
            }
        },
        Command::Scratch { command } => match command {
            ScratchCommand::Claim { session, change } => scratch::claim(project, &session, &change),
        },
    }
}

pub(crate) fn eprint_chain(error: &dyn std::error::Error) {
    eprintln!("error: {error}");
    let mut source = error.source();
    while let Some(cause) = source {
        eprintln!("  caused by: {cause}");
        source = cause.source();
    }
}
