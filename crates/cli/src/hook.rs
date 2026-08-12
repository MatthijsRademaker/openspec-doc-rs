use std::io::{self, Read};

use openspec_doc_core::Project;
use openspec_doc_core::comments::ScopeKey;
use openspec_doc_core::hook::{
    Agent, HookDecision, encode_decision, ensure_session, load_directive, load_pending,
    mark_consumed, parse_event, parse_session_id, start_explore, translate,
};
use openspec_doc_core::scratch::{self, Promotion};

use crate::discovery;
use crate::error::Error;
use crate::report;

/// The dashboard side of a turn boundary, so the hook's own logic can be tested
/// without starting real servers or opening real browser tabs.
trait Reviewable {
    /// The port a dashboard for `project` is serving on.
    fn ensure(&self, project: &Project) -> Result<u16, Error>;
    fn open(&self, url: &str);
}

/// The real thing: discover or start a dashboard, and open a real browser.
struct Live;

impl Reviewable for Live {
    fn ensure(&self, project: &Project) -> Result<u16, Error> {
        discovery::ensure(project)
    }

    fn open(&self, url: &str) {
        openspec_doc_server::open_browser(url);
    }
}

/// Handle an agent's turn-end hook: normalize its payload, promote the
/// session's scratch note if its exploration has been formalized, make sure a
/// dashboard is up for a session that has something to review, turn any
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

    register(&project, &event.session_id, &Live)?;

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

/// Handle an agent's prompt-submission event: deliver any standing directive as
/// context for the turn that prompt is about to start.
///
/// This is the same directive the turn-end hook injects, delivered one turn
/// earlier. The reviewer's own prompt is the one the review feedback belongs to,
/// and Stop is by construction too late for it. Whichever hook fires first
/// consumes the directive, so the other finds nothing pending.
///
/// Promotion is deliberately not checked here: it looks for change directories
/// that appear as a *result* of agent work, so a turn boundary is when there is
/// something new to find, and running it would put an `openspec validate`
/// subprocess in front of every prompt the reviewer types.
pub fn prompt(project: Project, agent: Agent) -> Result<(), Error> {
    // The one place in this project where a failure is reported instead of
    // propagated. `hook stop` can exit non-zero safely: the worst outcome there
    // is a turn that ends normally. This hook sits in front of the human's own
    // input, where a non-zero exit may refuse the prompt outright — leaving the
    // reviewer unable to type at all, which is strictly worse than the delayed
    // feedback this command exists to fix. Everything it emits is advisory
    // context, so delivering nothing is a real fallback rather than a fake
    // success.
    if let Err(error) = deliver(project, agent) {
        report(
            "could not deliver review feedback with this prompt; the prompt goes through without it",
        );
        crate::eprint_chain(&error);
    }

    Ok(())
}

/// Translate any standing verdict and emit the resulting directive's reason text
/// on stdout, which is what this event's contract adds to the model's context.
///
/// Emits nothing when the session has no untranslated verdict and no pending
/// directive, so an ordinary prompt adds no context.
fn deliver(project: Project, agent: Agent) -> Result<(), Error> {
    let mut payload = String::new();
    io::stdin()
        .read_to_string(&mut payload)
        .map_err(|source| Error::Stdin { source })?;

    let session_id = parse_session_id(agent, &payload)?;

    if let Some((key, verdict)) = translate(&project, &session_id)? {
        report(&format!(
            "injected the {verdict} verdict on {}",
            scope(&key)
        ));
    }

    // Consume before emitting, matching `stop`: a failed write must not deliver
    // a directive that would fire again at the next turn boundary.
    if let Some(directive) = load_pending(&project.root, &session_id)? {
        mark_consumed(&project.root, &session_id, &directive)?;
        println!("{}", directive.reason);
    }

    Ok(())
}

/// Handle the agent's explore-command event: create the session's scratch note
/// and print where it is, alongside the URL the review will appear at.
///
/// This command deliberately touches no dashboard. At the instant it returns the
/// note does not exist — readying its location is all that has happened — so the
/// session page would render no artifact and say "No artifact on disk to review
/// yet.", with nothing for the reviewer to select. One turn later the note is
/// there, which is where the dashboard and the browser belong.
///
/// The URL can still be given now, and that is what the port assignment buys: it
/// is a property of the project rather than of a running process, so it can be
/// named a full turn before anything has bound it.
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

    if let Some(port) = discovery::preferred(&project) {
        println!(
            "openspec-doc: this project's dashboard is at {}, and this exploration will be \
             reviewed at {}. It starts at the end of this turn; answer with that URL when asked \
             where the review is.",
            openspec_doc_core::dashboard::url(discovery::HOST, port),
            session_page(port, &session_id)
        );
    }

    Ok(())
}

/// Make this session visible to the reviewer, if it has anything for them: write
/// its directive record, make sure a dashboard is serving the project, and open
/// its page the first time round.
///
/// One predicate gates all three. Registering unconditionally makes a directive
/// record mean only that *some turn ended in this project*, so the index fills
/// with sessions that never explored, alongside the ones the reviewer is looking
/// for — and a session where the owner asked one question about one function
/// leaves a daemon behind.
fn register(project: &Project, session_id: &str, dashboard: &dyn Reviewable) -> Result<(), Error> {
    if !has_review_material(project, session_id)? {
        return Ok(());
    }

    // Read before registering: "was this session already registered" is the whole
    // of the browser's once-per-session bound, and one turn later the answer is
    // different. No flag is needed for it.
    let first_registration = load_directive(&project.root, session_id)?.is_none();

    // Registered before the verdict check: the reviewer cannot submit the verdict
    // this loop runs on until the dashboard can see the session, and the dashboard
    // sees a session by its directive record.
    ensure_session(&project.root, session_id)?;

    report_dashboard(project, session_id, first_registration, dashboard);

    Ok(())
}

/// Whether this session has something for the reviewer to look at.
///
/// One read covers both cases the requirement names. A session whose exploration
/// was promoted still has a note at its session path — promotion leaves a
/// redirect there — so "the note exists" is also "it was promoted", and a second
/// clause for the promoted case would be logic that can never fire.
fn has_review_material(project: &Project, session_id: &str) -> Result<bool, Error> {
    Ok(scratch::read_session(&project.root, session_id)?.is_some())
}

/// Make sure a dashboard is serving this project, and open the session's page the
/// once, reporting on stderr either way.
///
/// A failure to ensure one is reported and carried past, the way
/// [`report_promotion`] is: a dashboard that would not start must not cost the
/// session its review feedback.
fn report_dashboard(
    project: &Project,
    session_id: &str,
    first_registration: bool,
    dashboard: &dyn Reviewable,
) {
    let port = match dashboard.ensure(project) {
        Ok(port) => port,
        Err(error) => {
            report(&format!(
                "could not bring up a dashboard for this project; continuing to the verdict check. \
                 A failed start explains itself in {}",
                discovery::log_path(project).display()
            ));
            crate::eprint_chain(&error);

            return;
        }
    };

    // Bounded by first registration rather than by whether a dashboard was
    // *started*. The two agree on the common case and disagree on both of the
    // interesting ones: a dashboard already up for another session on this root is
    // still a dashboard whose page for *this* session is new, and a dashboard that
    // died and came back at a later boundary must not open a second tab.
    let page = session_page(port, session_id);
    if first_registration {
        report(&format!("opening this session's review page at {page}"));
        dashboard.open(&page);
    } else {
        report(&format!("this session is being reviewed at {page}"));
    }
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

/// The URL of `session_id`'s review page on the dashboard at `port`.
fn session_page(port: u16, session_id: &str) -> String {
    openspec_doc_core::dashboard::session_page(discovery::HOST, port, session_id)
}

/// How a scope is named in a diagnostic.
fn scope(key: &ScopeKey) -> String {
    match key {
        ScopeKey::Session(session_id) => format!("session {session_id}"),
        ScopeKey::Change(name) => format!("change {name}"),
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::fs;

    use openspec_doc_core::project_at;
    use tempfile::TempDir;

    use super::*;

    const SESSION: &str = "0199a4c6-3b2e-7c41-9f8d-2a6b5c1e0d74";
    const PORT: u16 = 4327;

    /// A dashboard that is always up, recording what it was asked for. Injected so
    /// these tests never bind a port or open a browser tab.
    #[derive(Default)]
    struct Fake {
        ensured: RefCell<usize>,
        opened: RefCell<Vec<String>>,
        unavailable: bool,
    }

    impl Reviewable for Fake {
        fn ensure(&self, _project: &Project) -> Result<u16, Error> {
            *self.ensured.borrow_mut() += 1;

            if self.unavailable {
                return Err(Error::NoAssignedPort);
            }

            Ok(PORT)
        }

        fn open(&self, url: &str) {
            self.opened.borrow_mut().push(url.to_owned());
        }
    }

    impl Fake {
        fn ensured(&self) -> usize {
            *self.ensured.borrow()
        }

        fn opened(&self) -> Vec<String> {
            self.opened.borrow().clone()
        }
    }

    fn fixture() -> (TempDir, Project) {
        let temp = TempDir::new().expect("temp dir");
        fs::create_dir_all(temp.path().join("openspec")).expect("create openspec dir");
        fs::write(temp.path().join("openspec/config.yaml"), "").expect("write config");
        let project = project_at(temp.path()).expect("project");
        (temp, project)
    }

    fn write_note(project: &Project) {
        let path = scratch::session_path(&project.root, SESSION).expect("note path");
        fs::create_dir_all(path.parent().expect("parent")).expect("create scratch dir");
        fs::write(path, "# Exploration\n\nSomething to read.\n").expect("write note");
    }

    fn registered(project: &Project) -> bool {
        load_directive(&project.root, SESSION)
            .expect("load directive")
            .is_some()
    }

    #[test]
    fn a_session_with_an_exploration_is_registered_and_gets_a_dashboard() {
        let (_temp, project) = fixture();
        write_note(&project);
        let dashboard = Fake::default();

        register(&project, SESSION, &dashboard).expect("register");

        assert!(registered(&project));
        assert_eq!(dashboard.ensured(), 1);
    }

    /// A session where the owner asked one question about one function must not
    /// leave a daemon behind, and must not show up in the reviewer's index.
    #[test]
    fn a_session_with_nothing_to_review_is_neither_registered_nor_given_a_dashboard() {
        let (_temp, project) = fixture();
        let dashboard = Fake::default();

        register(&project, SESSION, &dashboard).expect("register");

        assert!(!registered(&project));
        assert_eq!(dashboard.ensured(), 0);
        assert!(dashboard.opened().is_empty());
    }

    /// Promotion leaves a redirect at the session path, and a session whose
    /// exploration became a change is still a session under review.
    #[test]
    fn a_promoted_exploration_still_gets_a_dashboard() {
        let (_temp, project) = fixture();
        let path = scratch::session_path(&project.root, SESSION).expect("note path");
        fs::create_dir_all(path.parent().expect("parent")).expect("create scratch dir");
        fs::write(
            path,
            "<!-- openspec-doc:moved-to .openspec-doc/scratch/add-a.md -->\n",
        )
        .expect("write redirect");
        let dashboard = Fake::default();

        register(&project, SESSION, &dashboard).expect("register");

        assert_eq!(
            scratch::promoted_to(&project.root, SESSION).expect("promoted"),
            Some("add-a".to_owned()),
            "this fixture is meant to be a promoted session"
        );
        assert_eq!(dashboard.ensured(), 1);
    }

    /// The first turn boundary of an exploration is the first moment the page has
    /// the note on it. Every later one finds the record already there.
    #[test]
    fn the_browser_opens_once_at_this_sessions_page_and_never_again() {
        let (_temp, project) = fixture();
        write_note(&project);
        let dashboard = Fake::default();

        register(&project, SESSION, &dashboard).expect("first turn boundary");
        register(&project, SESSION, &dashboard).expect("second turn boundary");
        register(&project, SESSION, &dashboard).expect("third turn boundary");

        assert_eq!(
            dashboard.opened(),
            [format!("http://127.0.0.1:{PORT}/sessions/{SESSION}")],
            "the tab must open once, at this session's own page"
        );
        assert_eq!(
            dashboard.ensured(),
            3,
            "every turn boundary is a heartbeat, whether or not it opens a tab"
        );
    }

    /// Bounded by first registration rather than by whether a dashboard was
    /// started: a session already registered opens nothing however the dashboard
    /// got there.
    #[test]
    fn a_session_already_registered_opens_no_second_tab() {
        let (_temp, project) = fixture();
        write_note(&project);
        ensure_session(&project.root, SESSION).expect("register by hand");
        let dashboard = Fake::default();

        register(&project, SESSION, &dashboard).expect("register");

        assert!(dashboard.opened().is_empty());
    }

    /// A dashboard that would not come up must not cost the session its review
    /// feedback, so the failure is reported and the turn boundary carries on.
    #[test]
    fn a_dashboard_that_cannot_be_ensured_is_reported_and_carried_past() {
        let (_temp, project) = fixture();
        write_note(&project);
        let dashboard = Fake {
            unavailable: true,
            ..Fake::default()
        };

        register(&project, SESSION, &dashboard).expect("a failed ensure must not propagate");

        assert!(registered(&project), "the session is still registered");
        assert!(
            dashboard.opened().is_empty(),
            "a tab at a dead URL is worse than the reported failure"
        );
    }
}
