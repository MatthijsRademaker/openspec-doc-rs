use std::io::{self, Read};

use openspec_doc_core::Project;
use openspec_doc_core::comments::ScopeKey;
use openspec_doc_core::hook::{
    Agent, HookDecision, encode_decision, ensure_session, load_pending, mark_consumed, parse_event,
    parse_session_id, start_explore, translate,
};
use openspec_doc_core::scratch::{self, Promotion};

use crate::error::Error;

/// Handle an agent's turn-end hook: normalize its payload, promote the
/// session's scratch note if its exploration has been formalized, turn any
/// standing review verdict into a directive, then resolve and emit that agent's
/// stop decision.
///
/// Errors propagate: the process exits non-zero having emitted no decision.
/// Neither agent treats that as "block" — Claude Code surfaces the hook's
/// stderr and ends the turn, and our pi extension injects nothing — so a bug
/// here fails loudly without wedging the session.
pub fn stop(project: Project, agent: Agent) -> Result<(), Error> {
    let mut payload = String::new();
    io::stdin()
        .read_to_string(&mut payload)
        .map_err(|source| Error::Stdin { source })?;

    let event = parse_event(agent, &payload)?;

    // Registered first, and before anything that can fail: the reviewer cannot
    // submit the verdict this loop runs on until the dashboard can see the
    // session, and the dashboard sees a session by its directive record.
    ensure_session(&project.root, &event.session_id)?;

    // Promotion runs before translation because a move-to-proposal directive
    // injected at the last turn boundary is precisely what produces the claim
    // this check now has to notice.
    report_promotion(&project, &event.session_id);

    if let Some((key, verdict)) = translate(&project, &event.session_id)? {
        report(&format!(
            "injected the {verdict} verdict on {}",
            scope(&key)
        ));
    }

    // Consume before emitting, so a failed write can never inject a directive
    // that would fire again at the next turn boundary.
    let decision = match load_pending(&project.root, &event.session_id)? {
        Some(directive) => {
            mark_consumed(&project.root, &event.session_id, &directive)?;
            HookDecision::Block {
                reason: directive.reason,
            }
        }
        None => HookDecision::Allow,
    };

    println!("{}", encode_decision(agent, &decision));

    Ok(())
}

/// Handle the agent's explore-command event: create the session's scratch note
/// and print where it is.
///
/// The instruction goes to stdout because that is what this event's contract
/// adds to the model's context. Only the session id is read from the payload,
/// so nothing here depends on fields specific to one event shape.
pub fn explore(project: Project, agent: Agent) -> Result<(), Error> {
    let mut payload = String::new();
    io::stdin()
        .read_to_string(&mut payload)
        .map_err(|source| Error::Stdin { source })?;

    let session_id = parse_session_id(agent, &payload)?;

    println!("{}", start_explore(&project.root, &session_id)?);

    Ok(())
}

/// Run the session's promotion check and report what it did on stderr, which is
/// where both agents surface a hook's diagnostics.
///
/// A failed check is reported rather than propagated: the verdict this same
/// invocation may have to translate is independent of it, and losing that too
/// would turn one recoverable problem into a stalled review loop.
fn report_promotion(project: &Project, session_id: &str) {
    match scratch::check(project, session_id) {
        Ok(Promotion::NotPromoted) => {}
        Ok(Promotion::Promoted {
            change,
            note_path,
            validation,
        }) => {
            report(&format!(
                "promoted the scratch note to {change}, now at {}",
                note_path.display()
            ));
            report(&format!(
                "`openspec validate {change}` {}",
                if validation.passed {
                    "passed"
                } else {
                    "FAILED"
                }
            ));
            if !validation.passed {
                eprint!("{}", validation.output);
            }
        }
        Err(error) => {
            report("promotion check failed; continuing to the verdict check");
            crate::eprint_chain(&error);
        }
    }
}

/// How a scope is named in a diagnostic.
fn scope(key: &ScopeKey) -> String {
    match key {
        ScopeKey::Session(session_id) => format!("session {session_id}"),
        ScopeKey::Change(name) => format!("change {name}"),
    }
}

/// Diagnostics go to stderr; stdout carries the decision the agent parses.
fn report(message: &str) {
    eprintln!("openspec-doc: {message}");
}
