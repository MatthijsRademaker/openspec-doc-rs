//! The agent hook bridge: normalize a coding agent's turn-end event, resolve
//! the session's pending directive, and emit that agent's stop decision.

mod adapter;
mod directive;
mod event;

pub use adapter::{encode_decision, parse_event};
pub use directive::{
    Directive, load_pending, mark_consumed, path as directive_path, sessions, sessions_dir,
};
pub use event::{Agent, HookDecision, HookEvent};
