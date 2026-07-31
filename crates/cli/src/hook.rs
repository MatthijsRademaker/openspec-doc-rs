use std::io::{self, Read};

use openspec_doc_core::Project;
use openspec_doc_core::hook::{
    Agent, HookDecision, encode_decision, load_pending, mark_consumed, parse_event,
};

use crate::error::Error;

/// Handle an agent's turn-end hook: normalize its payload, resolve the
/// session's pending directive, and emit that agent's stop decision.
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
