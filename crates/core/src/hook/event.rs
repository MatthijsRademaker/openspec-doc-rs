use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use crate::error::Error;

/// The coding agent a hook invocation came from. Selects the wire format used
/// to parse the incoming payload and serialize the outgoing decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Agent {
    Claude,
    Pi,
}

impl Agent {
    pub const VALUES: [&'static str; 2] = ["claude", "pi"];

    fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Pi => "pi",
        }
    }
}

impl fmt::Display for Agent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Agent {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "claude" => Ok(Self::Claude),
            "pi" => Ok(Self::Pi),
            _ => Err(Error::UnknownAgent {
                value: value.to_owned(),
            }),
        }
    }
}

/// A turn-end ("stop") event, normalized across agents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookEvent {
    pub agent: Agent,
    pub session_id: String,
    pub transcript_path: PathBuf,
    pub cwd: PathBuf,
}

/// What the agent should do at the turn boundary it is asking about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookDecision {
    /// Let the turn end.
    Allow,
    /// Keep the session alive and feed `reason` back to the agent.
    Block { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_round_trips_through_its_cli_spelling() {
        for agent in [Agent::Claude, Agent::Pi] {
            assert_eq!(agent.to_string().parse::<Agent>().unwrap(), agent);
        }
    }

    #[test]
    fn unknown_agent_names_are_rejected() {
        let error = "cursor".parse::<Agent>().expect_err("unknown agent");

        assert!(matches!(error, Error::UnknownAgent { .. }));
        assert!(error.to_string().contains("claude"), "{error}");
    }
}
