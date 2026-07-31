//! The agent-specific wire formats on the hook's stdin and stdout.
//!
//! Claude Code defines its own Stop-hook schema, which we must match exactly.
//! pi.dev has no external-process hook at all — its turn boundary is only
//! reachable from a TypeScript extension (`pi.on("agent_end")`), so the pi
//! format is one we define for that extension to speak. It is deliberately
//! *not* identical to Claude Code's: pointing `--agent pi` at a Claude payload
//! (or the reverse) then fails loudly instead of half-working.

use std::path::PathBuf;

use serde::Deserialize;
use serde_json::json;

use crate::error::Error;
use crate::hook::event::{Agent, HookDecision, HookEvent};

/// Claude Code's Stop-hook payload. Its other fields (`hook_event_name`,
/// `stop_hook_active`, ...) are ignored.
#[derive(Debug, Deserialize)]
struct ClaudePayload {
    session_id: String,
    transcript_path: PathBuf,
    cwd: PathBuf,
}

/// The payload our pi.dev extension builds from `ctx.sessionManager`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PiPayload {
    session_id: String,
    transcript_path: PathBuf,
    cwd: PathBuf,
}

/// Normalize `agent`'s stdin payload into a [`HookEvent`].
pub fn parse_event(agent: Agent, payload: &str) -> Result<HookEvent, Error> {
    let (session_id, transcript_path, cwd) = match agent {
        Agent::Claude => {
            let payload: ClaudePayload = parse(agent, payload)?;
            (payload.session_id, payload.transcript_path, payload.cwd)
        }
        Agent::Pi => {
            let payload: PiPayload = parse(agent, payload)?;
            (payload.session_id, payload.transcript_path, payload.cwd)
        }
    };

    Ok(HookEvent {
        agent,
        session_id,
        transcript_path,
        cwd,
    })
}

/// Serialize `decision` in the shape `agent` expects on the hook's stdout.
pub fn encode_decision(agent: Agent, decision: &HookDecision) -> String {
    let value = match (agent, decision) {
        (Agent::Claude, HookDecision::Block { reason }) => {
            json!({ "decision": "block", "reason": reason })
        }
        (Agent::Claude, HookDecision::Allow) => json!({ "continue": true }),
        (Agent::Pi, HookDecision::Block { reason }) => {
            json!({ "action": "continue", "message": reason })
        }
        (Agent::Pi, HookDecision::Allow) => json!({ "action": "stop" }),
    };

    serde_json::to_string(&value).expect("a decision built from strings is serializable")
}

fn parse<T: for<'de> Deserialize<'de>>(agent: Agent, payload: &str) -> Result<T, Error> {
    serde_json::from_str(payload).map_err(|source| Error::HookPayload { agent, source })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Captured verbatim from a real Claude Code Stop hook (only the session id
    /// and paths are rewritten). Claude Code sends considerably more than we
    /// read, so keep every field: this is the regression test that an added
    /// upstream field does not become a parse error.
    const CLAUDE_STOP: &str = r#"{
      "session_id": "0199a4c6-3b2e-7c41-9f8d-2a6b5c1e0d74",
      "transcript_path": "/home/dev/.claude/projects/-home-dev-proj/0199a4c6.jsonl",
      "cwd": "/home/dev/proj",
      "prompt_id": "690a2a4b-92b0-43f7-8625-b1ab6de3b854",
      "permission_mode": "default",
      "effort": { "level": "high" },
      "hook_event_name": "Stop",
      "stop_hook_active": true,
      "last_assistant_message": "Done.",
      "background_tasks": [],
      "session_crons": []
    }"#;

    /// The payload our pi.dev extension emits, captured from a real pi
    /// `agent_end` invocation of `.pi/extensions/openspec-doc-hook.ts`.
    const PI_STOP: &str = r#"{
      "sessionId": "019fb777-10e8-7e63-9a82-7632a31bc124",
      "transcriptPath": "/home/dev/.pi/agent/sessions/--home-dev-proj--/2026-07-31T09-17-41-480Z_019fb777-10e8-7e63-9a82-7632a31bc124.jsonl",
      "cwd": "/home/dev/proj"
    }"#;

    #[test]
    fn claude_payload_is_normalized() {
        let event = parse_event(Agent::Claude, CLAUDE_STOP).expect("claude payload");
        // `stop_hook_active: true` marks a turn Claude Code is only running
        // because a Stop hook blocked the previous one. We deliberately ignore
        // it: re-fire prevention is the directive's consume-once, not this flag.

        assert_eq!(event.agent, Agent::Claude);
        assert_eq!(event.session_id, "0199a4c6-3b2e-7c41-9f8d-2a6b5c1e0d74");
        assert_eq!(
            event.transcript_path,
            PathBuf::from("/home/dev/.claude/projects/-home-dev-proj/0199a4c6.jsonl")
        );
        assert_eq!(event.cwd, PathBuf::from("/home/dev/proj"));
    }

    #[test]
    fn pi_payload_is_normalized() {
        let event = parse_event(Agent::Pi, PI_STOP).expect("pi payload");

        assert_eq!(event.agent, Agent::Pi);
        assert_eq!(event.session_id, "019fb777-10e8-7e63-9a82-7632a31bc124");
        assert_eq!(
            event.transcript_path,
            PathBuf::from(
                "/home/dev/.pi/agent/sessions/--home-dev-proj--/2026-07-31T09-17-41-480Z_019fb777-10e8-7e63-9a82-7632a31bc124.jsonl"
            )
        );
        assert_eq!(event.cwd, PathBuf::from("/home/dev/proj"));
    }

    #[test]
    fn each_agent_rejects_the_other_agents_payload() {
        assert!(matches!(
            parse_event(Agent::Claude, PI_STOP),
            Err(Error::HookPayload { .. })
        ));
        assert!(matches!(
            parse_event(Agent::Pi, CLAUDE_STOP),
            Err(Error::HookPayload { .. })
        ));
    }

    #[test]
    fn malformed_payloads_are_rejected() {
        for payload in ["", "not json", "{}", r#"{"session_id": 7}"#] {
            assert!(
                matches!(
                    parse_event(Agent::Claude, payload),
                    Err(Error::HookPayload { .. })
                ),
                "accepted {payload:?}"
            );
        }
    }

    #[test]
    fn claude_decisions_use_claude_codes_stop_hook_shape() {
        let block = HookDecision::Block {
            reason: "Address the open comments.".to_owned(),
        };

        assert_eq!(
            encode_decision(Agent::Claude, &block),
            r#"{"decision":"block","reason":"Address the open comments."}"#
        );
        assert_eq!(
            encode_decision(Agent::Claude, &HookDecision::Allow),
            r#"{"continue":true}"#
        );
    }

    #[test]
    fn pi_decisions_use_the_shape_our_extension_reads() {
        let block = HookDecision::Block {
            reason: "Address the open comments.".to_owned(),
        };

        assert_eq!(
            encode_decision(Agent::Pi, &block),
            r#"{"action":"continue","message":"Address the open comments."}"#
        );
        assert_eq!(
            encode_decision(Agent::Pi, &HookDecision::Allow),
            r#"{"action":"stop"}"#
        );
    }
}
