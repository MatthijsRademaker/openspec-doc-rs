//! The agent hook bridge: normalize a coding agent's turn-end event, resolve
//! the session's pending directive, and emit that agent's stop decision.

mod adapter;
mod directive;
mod event;
mod explore;
mod translate;

pub use adapter::{encode_decision, parse_event, parse_session_id};
pub use directive::{
    Directive, ensure_session, load as load_directive, load_pending, mark_consumed,
    path as directive_path, sessions, sessions_dir, write_pending,
};
pub use event::{Agent, HookDecision, HookEvent};
pub use explore::start as start_explore;
pub use translate::translate;
