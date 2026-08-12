//! The `openspec-doc` binary: argument parsing, shared project-root
//! resolution, and subcommand dispatch.

mod cli;
mod comment;
mod discovery;
mod error;
mod hook;
mod root;
mod scratch;
mod serve;
mod summary;

use std::process::ExitCode;

use clap::Parser;
use openspec_doc_core::comments::Status;

use crate::cli::{Cli, Command, CommentCommand, HookCommand, ScratchCommand, ServeCommand};
use crate::error::Error;

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => return usage_exit(&error),
    };

    // `hook prompt` sits in front of the human's own input, where a non-zero exit
    // may refuse the prompt outright. `hook::prompt` already swallows its own
    // failures, but root resolution happens out here, before dispatch — so
    // without this the guarantee has a hole the command itself cannot close.
    let fail_soft = matches!(
        cli.command,
        Command::Hook {
            command: HookCommand::Prompt { .. }
        }
    );

    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprint_chain(&error);
            if fail_soft {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
    }
}

/// What to exit with when the arguments themselves did not parse.
///
/// Clap's own `Error::exit` uses 2 for a usage error, and Claude Code treats a
/// `UserPromptSubmit` hook exiting 2 as a block: the reviewer's prompt is refused
/// and the session becomes unusable until the config is fixed. A typo in a hook
/// command string is therefore the one usage error that must not be reported the
/// usual way.
///
/// The intent is read back off the raw arguments because parsing is what failed,
/// so there is no parsed command to match on. A typo in `prompt` itself is not
/// recoverable here and still exits 2.
fn usage_exit(error: &clap::Error) -> ExitCode {
    let _ = error.print();

    match error.kind() {
        clap::error::ErrorKind::DisplayHelp
        | clap::error::ErrorKind::DisplayVersion
        | clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => ExitCode::SUCCESS,
        _ if prompt_hook_intended() => ExitCode::SUCCESS,
        _ => ExitCode::from(2),
    }
}

/// Whether this invocation was reaching for `hook prompt`, however badly.
fn prompt_hook_intended() -> bool {
    let args: Vec<String> = std::env::args().collect();
    args.windows(2)
        .any(|pair| pair[0] == "hook" && pair[1] == "prompt")
}

fn run(cli: Cli) -> Result<(), Error> {
    let project = root::resolve(cli.root.as_deref())?;

    match cli.command {
        Command::Summary => summary::run(project),
        Command::Serve {
            command: Some(ServeCommand::Url),
            ..
        } => serve::url(project),
        Command::Serve {
            command: None,
            host,
            port,
            no_open,
            idle_exit,
        } => serve::run(project, host, port, no_open, idle_exit),
        Command::Hook { command } => match command {
            HookCommand::Stop { agent } => hook::stop(project, agent),
            HookCommand::Prompt { agent } => hook::prompt(project, agent),
            HookCommand::Explore { agent } => hook::explore(project, agent),
        },
        Command::Comment { command } => match command {
            CommentCommand::Add {
                scope,
                artifact,
                selected_text,
                body,
            } => comment::add(
                project,
                scope.key(),
                artifact.as_deref(),
                selected_text.as_deref(),
                &body,
            ),
            CommentCommand::List { scope } => comment::list(project, scope.key()),
            CommentCommand::Reply {
                scope,
                comment,
                body,
            } => comment::reply(project, scope.key(), &comment, &body),
            CommentCommand::Edit {
                scope,
                comment,
                body,
            } => comment::edit(project, scope.key(), &comment, &body),
            CommentCommand::Address { scope, comment } => {
                comment::set_status(project, scope.key(), &comment, Status::Addressed)
            }
            CommentCommand::Resolve { scope, comment } => {
                comment::set_status(project, scope.key(), &comment, Status::Resolved)
            }
            CommentCommand::Reopen { scope, comment } => {
                comment::set_status(project, scope.key(), &comment, Status::Open)
            }
        },
        Command::Scratch { command } => match command {
            ScratchCommand::Claim { session, change } => scratch::claim(project, &session, &change),
        },
    }
}

/// Diagnostics go to stderr; stdout carries what the agent parses.
pub(crate) fn report(message: &str) {
    eprintln!("openspec-doc: {message}");
}

pub(crate) fn eprint_chain(error: &dyn std::error::Error) {
    eprintln!("error: {error}");
    let mut source = error.source();
    while let Some(cause) = source {
        eprintln!("  caused by: {cause}");
        source = cause.source();
    }
}
