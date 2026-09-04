//! End-to-end tests driving the built `openspec-doc` binary.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use openspec_doc_core::dashboard::STATE_DIR_ENV;
use tempfile::TempDir;

/// A port registry per test, since the harness gives each test its own thread.
///
/// Two reasons, and the second is the subtle one. Without an override at all,
/// every invocation here writes a real assignment for a temp directory into the
/// developer's own registry. And sharing one override across tests is not enough:
/// the registry is read-modify-write and deliberately unlocked — the design
/// accepts that a concurrent write costs one restart's worth of stability — so a
/// test asserting that a project's port is stable has to be the only writer.
fn state_dir() -> PathBuf {
    thread_local! {
        static STATE: TempDir = TempDir::new().expect("state dir");
    }

    STATE.with(|state| state.path().to_owned())
}

/// Start `openspec-doc` with this test's own port registry.
fn binary() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_openspec-doc"));
    command.env(STATE_DIR_ENV, state_dir());
    command
}

/// A temp directory that is a valid OpenSpec project root, containing each of
/// `dirs` (relative paths).
fn project_fixture(dirs: &[&str]) -> TempDir {
    let temp = TempDir::new().expect("temp dir");
    for dir in dirs {
        fs::create_dir_all(temp.path().join(dir)).expect("create fixture dir");
    }
    fs::create_dir_all(temp.path().join("openspec")).expect("create openspec dir");
    fs::write(temp.path().join("openspec/config.yaml"), "").expect("write config");

    // A turn boundary in a session that has an exploration ensures a dashboard,
    // and these tests drive the real binary, so there is nothing to inject: left
    // alone they leave a detached server on a real port serving a temp directory
    // that is about to be deleted, and open a browser tab per test. A directory
    // where the log belongs makes the start fail before it spawns anything, which
    // the hook reports and carries past — which is itself the behaviour task 5.7
    // asks for. The dashboard step is tested where it can be injected, in
    // `hook::tests` and `discovery::tests`.
    fs::create_dir_all(temp.path().join(".openspec-doc/serve.log")).expect("block the serve log");

    temp
}

fn run(args: &[&str]) -> Output {
    binary().args(args).output().expect("run openspec-doc")
}

fn run_in(cwd: &Path, args: &[&str]) -> Output {
    binary()
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("run openspec-doc")
}

fn run_with_stdin(args: &[&str], stdin: &str) -> Output {
    let mut child = binary()
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn openspec-doc");
    child
        .stdin
        .take()
        .expect("piped stdin")
        .write_all(stdin.as_bytes())
        .expect("write stdin");
    child.wait_with_output().expect("run openspec-doc")
}

/// Like `run_with_stdin`, but tolerant of the child exiting before it reads.
///
/// Argument parsing and root resolution both happen before stdin is touched, so
/// for an invocation that fails there the write races the child's exit: it
/// succeeds if the bytes fit the pipe buffer first and returns a broken pipe if
/// not. Either is correct, and `run_with_stdin` stays strict because for every
/// other test a failed write means the binary died when it should have been
/// reading.
fn run_expecting_early_exit(args: &[&str], stdin: &str) -> Output {
    let mut child = binary()
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn openspec-doc");
    let _ = child
        .stdin
        .take()
        .expect("piped stdin")
        .write_all(stdin.as_bytes());
    child.wait_with_output().expect("run openspec-doc")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("utf-8 stdout")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("utf-8 stderr")
}

#[test]
fn summary_reports_discovered_project_structure() {
    let fixture = project_fixture(&[
        "openspec/changes/add-b",
        "openspec/changes/add-a",
        "openspec/changes/archive/2026-01-01-add-old",
        "openspec/specs/project-scanner",
    ]);
    let root = fixture.path().canonicalize().expect("canonical root");

    let output = run(&["summary", "--root", root.to_str().unwrap()]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let stdout = stdout(&output);
    assert!(
        stdout.contains(&format!("root: {}", root.display())),
        "missing resolved root in:\n{stdout}"
    );
    assert!(stdout.contains("active changes (2):"), "{stdout}");
    assert!(stdout.contains("add-a"), "{stdout}");
    assert!(stdout.contains("add-b"), "{stdout}");
    assert!(stdout.contains("archived changes (1):"), "{stdout}");
    assert!(stdout.contains("2026-01-01-add-old"), "{stdout}");
    assert!(stdout.contains("specs (1):"), "{stdout}");
    assert!(stdout.contains("project-scanner"), "{stdout}");
}

#[test]
fn explicit_root_is_used_without_walking_up() {
    let fixture = project_fixture(&["openspec/changes/add-a", "nested/deeper"]);
    let root = fixture.path().canonicalize().expect("canonical root");
    let nested = root.join("nested/deeper");

    // The nested directory is not itself a project root, so `--root` must not
    // fall back to walking up to the fixture root.
    let output = run_in(&nested, &["summary", "--root", nested.to_str().unwrap()]);

    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("not an OpenSpec project root"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn missing_project_is_reported_without_a_panic() {
    let dir = TempDir::new().expect("temp dir");

    let output = run_in(dir.path(), &["summary"]);

    assert!(!output.status.success());
    let stderr = stderr(&output);
    assert!(
        stderr.starts_with("error: no OpenSpec project found"),
        "{stderr}"
    );
    assert!(!stderr.contains("panicked"), "{stderr}");
}

const ARTIFACT: &str = "openspec/changes/add-thing/proposal.md";
const MARKDOWN: &str = "## Why\n\nAlpha before. Selected sentence. Omega after.\n";
const SELECTED: &str = "Selected sentence.";

/// A project fixture holding `ARTIFACT` with `MARKDOWN` in it.
fn project_with_artifact() -> TempDir {
    let fixture = project_fixture(&["openspec/changes/add-thing"]);
    fs::write(fixture.path().join(ARTIFACT), MARKDOWN).expect("write artifact");
    fixture
}

/// A `comment <action>` invocation scoped to `project_with_artifact`'s change,
/// with `extra` appended.
fn comment_args<'a>(action: &'a str, root: &'a str, extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec!["comment", action, "--root", root, "--change", "add-thing"];
    args.extend_from_slice(extra);
    args
}

/// The arguments that anchor a comment to `SELECTED`.
const ADD_ARGS: [&str; 6] = [
    "--artifact",
    ARTIFACT,
    "--selected-text",
    SELECTED,
    "--body",
    "Needs a rationale.",
];

/// The id `comment add` reported, taken from its `added comment <id>` line.
fn added_id(output: &Output) -> String {
    stdout(output)
        .lines()
        .next()
        .expect("an output line")
        .strip_prefix("added comment ")
        .expect("an `added comment <id>` line")
        .to_owned()
}

#[test]
fn comment_add_anchors_a_comment_that_list_then_reports() {
    let fixture = project_with_artifact();
    let root = fixture.path().to_str().unwrap().to_owned();

    let added = run(&comment_args("add", &root, &ADD_ARGS));

    assert!(added.status.success(), "{}", stderr(&added));
    let id = added_id(&added);
    assert!(
        fixture
            .path()
            .join(".openspec-doc/comments/add-thing.jsonl")
            .is_file(),
        "the sidecar is written at the change key"
    );

    let listed = run(&comment_args("list", &root, &[]));

    assert!(listed.status.success(), "{}", stderr(&listed));
    let stdout = stdout(&listed);
    assert!(stdout.contains("comments (1):"), "{stdout}");
    assert!(stdout.contains(&id), "{stdout}");
    assert!(stdout.contains("[open]"), "{stdout}");
    assert!(stdout.contains("anchor: exact"), "{stdout}");
    assert!(stdout.contains("Needs a rationale."), "{stdout}");
}

#[test]
fn comment_reply_and_resolve_show_up_in_the_next_list() {
    let fixture = project_with_artifact();
    let root = fixture.path().to_str().unwrap().to_owned();
    let id = added_id(&run(&comment_args("add", &root, &ADD_ARGS)));

    let replied = run(&comment_args(
        "reply",
        &root,
        &["--comment", &id, "--body", "Because of X."],
    ));
    let resolved = run(&comment_args("resolve", &root, &["--comment", &id]));

    assert!(replied.status.success(), "{}", stderr(&replied));
    assert!(resolved.status.success(), "{}", stderr(&resolved));
    let stdout = stdout(&run(&comment_args("list", &root, &[])));
    assert!(stdout.contains("[resolved]"), "{stdout}");
    assert!(stdout.contains("reply: Because of X."), "{stdout}");
}

/// The agent claims `addressed`, the reviewer grants `resolved`, and rejecting
/// the response puts the same thread back to `open` rather than making a new one.
#[test]
fn comment_address_and_reopen_move_the_thread_the_list_reports() {
    let fixture = project_with_artifact();
    let root = fixture.path().to_str().unwrap().to_owned();
    let id = added_id(&run(&comment_args("add", &root, &ADD_ARGS)));
    let listed_status = || {
        let stdout = stdout(&run(&comment_args("list", &root, &[])));
        assert!(stdout.contains("comments (1):"), "{stdout}");
        stdout
    };

    for (action, expected) in [
        ("address", "[addressed]"),
        ("resolve", "[resolved]"),
        ("reopen", "[open]"),
    ] {
        let output = run(&comment_args(action, &root, &["--comment", &id]));

        assert!(output.status.success(), "{}", stderr(&output));
        let stdout = listed_status();
        assert!(stdout.contains(expected), "after {action}: {stdout}");
    }
}

#[test]
fn a_status_change_naming_an_unknown_comment_fails_loudly() {
    let fixture = project_with_artifact();
    let root = fixture.path().to_str().unwrap().to_owned();
    run(&comment_args("add", &root, &ADD_ARGS));

    for action in ["address", "resolve", "reopen"] {
        let output = run(&comment_args(
            action,
            &root,
            &["--comment", "not-a-comment-id"],
        ));

        assert!(!output.status.success(), "{action} accepted a ghost id");
        assert!(
            stderr(&output).contains("not-a-comment-id"),
            "{}",
            stderr(&output)
        );
    }
}

/// The reviewer's feedback about the scope as a whole. It goes in the same
/// sidecar as every anchored comment, so a directive points the agent at one file.
#[test]
fn comment_add_without_a_selection_records_an_unanchored_comment() {
    let fixture = project_with_artifact();
    let root = fixture.path().to_str().unwrap().to_owned();

    let added = run(&comment_args(
        "add",
        &root,
        &["--body", "The whole framing is off."],
    ));

    assert!(added.status.success(), "{}", stderr(&added));
    let id = added_id(&added);
    let listed = stdout(&run(&comment_args("list", &root, &[])));
    assert!(listed.contains(&id), "{listed}");
    assert!(listed.contains("anchor: unanchored"), "{listed}");
    assert!(
        !listed.contains("selected:"),
        "an unanchored comment quotes nothing: {listed}"
    );
    assert!(listed.contains("The whole framing is off."), "{listed}");
}

/// Half a selection is a mistake, not a scope-level comment: `--artifact`
/// without the text to find in it would silently become unanchored feedback.
#[test]
fn comment_add_refuses_an_artifact_without_the_text_it_selects() {
    let fixture = project_with_artifact();
    let root = fixture.path().to_str().unwrap().to_owned();

    for half in [
        vec!["--artifact", ARTIFACT, "--body", "Body."],
        vec!["--selected-text", SELECTED, "--body", "Body."],
    ] {
        let output = run(&comment_args("add", &root, &half));

        assert!(!output.status.success(), "accepted {half:?}");
        assert!(
            stderr(&output).contains("--artifact") && stderr(&output).contains("--selected-text"),
            "{}",
            stderr(&output)
        );
    }
}

/// A reviewer who mistypes should not have to leave the agent two instructions
/// and no way to tell which one stands.
#[test]
fn comment_edit_replaces_the_body_the_next_list_reports() {
    let fixture = project_with_artifact();
    let root = fixture.path().to_str().unwrap().to_owned();
    let id = added_id(&run(&comment_args("add", &root, &ADD_ARGS)));

    let edited = run(&comment_args(
        "edit",
        &root,
        &["--comment", &id, "--body", "Needs a rationale, not a link."],
    ));

    assert!(edited.status.success(), "{}", stderr(&edited));
    let listed = stdout(&run(&comment_args("list", &root, &[])));
    assert!(listed.contains("comments (1):"), "{listed}");
    assert!(
        listed.contains("Needs a rationale, not a link."),
        "{listed}"
    );
    assert!(
        !listed.contains("Needs a rationale.\n"),
        "the superseded body is still reported: {listed}"
    );
    // The sidecar is append-only: the body first written stays on the record.
    let sidecar = fs::read_to_string(
        fixture
            .path()
            .join(".openspec-doc/comments/add-thing.jsonl"),
    )
    .expect("read sidecar");
    assert_eq!(sidecar.lines().count(), 2, "{sidecar}");
    assert!(sidecar.contains("Needs a rationale.\""), "{sidecar}");
}

#[test]
fn comment_edit_naming_an_unknown_comment_fails_loudly() {
    let fixture = project_with_artifact();
    let root = fixture.path().to_str().unwrap().to_owned();
    run(&comment_args("add", &root, &ADD_ARGS));

    let output = run(&comment_args(
        "edit",
        &root,
        &["--comment", "not-a-comment-id", "--body", "New body."],
    ));

    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("not-a-comment-id"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn comment_list_reports_a_moved_anchor_as_fuzzy_rather_than_exact() {
    let fixture = project_with_artifact();
    let root = fixture.path().to_str().unwrap().to_owned();
    run(&comment_args("add", &root, &ADD_ARGS));

    // Prepend a paragraph, pushing the anchored sentence off its recorded offset.
    fs::write(
        fixture.path().join(ARTIFACT),
        MARKDOWN.replace("## Why\n\n", "## Why\n\nNew introduction.\n\n"),
    )
    .expect("rewrite artifact");

    let stdout = stdout(&run(&comment_args("list", &root, &[])));

    assert!(stdout.contains("anchor: fuzzy"), "{stdout}");
    assert!(!stdout.contains("anchor: exact"), "{stdout}");
}

#[test]
fn comment_add_against_absent_selected_text_fails_loudly() {
    let fixture = project_with_artifact();
    let root = fixture.path().to_str().unwrap().to_owned();

    let output = run(&comment_args(
        "add",
        &root,
        &[
            "--artifact",
            ARTIFACT,
            "--selected-text",
            "never written",
            "--body",
            "Needs a rationale.",
        ],
    ));

    assert!(!output.status.success());
    assert!(
        stderr(&output).starts_with("error: selected text \"never written\" was not found in"),
        "{}",
        stderr(&output)
    );
    assert!(
        !fixture.path().join(".openspec-doc/comments").exists(),
        "a rejected comment writes no sidecar"
    );
}

#[test]
fn comment_requires_exactly_one_scope_key() {
    let fixture = project_with_artifact();
    let root = fixture.path().to_str().unwrap().to_owned();

    for scope in [
        vec!["comment", "list", "--root", &root],
        vec![
            "comment",
            "list",
            "--root",
            &root,
            "--change",
            "add-thing",
            "--session",
            "session-a",
        ],
    ] {
        let output = run(&scope);

        assert!(!output.status.success(), "accepted {scope:?}");
        assert!(
            stderr(&output).contains("--change") && stderr(&output).contains("--session"),
            "{}",
            stderr(&output)
        );
    }
}

const SESSION_ID: &str = "0199a4c6-3b2e-7c41-9f8d-2a6b5c1e0d74";
const REASON: &str = "Open comments remain. Address them.";

/// Per agent: the turn-end payload it sends on stdin, the block decision it
/// expects for `REASON`, and its allow decision.
const AGENTS: [(&str, &str, &str, &str); 2] = [
    (
        "claude",
        r#"{"session_id":"SESSION","transcript_path":"/t.jsonl","cwd":"/x","hook_event_name":"Stop","stop_hook_active":false}"#,
        r#"{"decision":"block","reason":"Open comments remain. Address them."}"#,
        r#"{"continue":true}"#,
    ),
    (
        "pi",
        r#"{"sessionId":"SESSION","transcriptPath":"/t.jsonl","cwd":"/x"}"#,
        r#"{"action":"continue","message":"Open comments remain. Address them."}"#,
        r#"{"action":"stop"}"#,
    ),
];

fn directive_path(root: &Path, session_id: &str) -> PathBuf {
    root.join(".openspec-doc/directives/_session")
        .join(format!("{session_id}.json"))
}

fn write_pending_directive(root: &Path, session_id: &str, reason: &str) {
    let path = directive_path(root, session_id);
    fs::create_dir_all(path.parent().unwrap()).expect("create directives dir");
    fs::write(
        &path,
        format!(
            r#"{{"pending":true,"reason":"{reason}","createdAt":"2026-07-31T08:00:00Z","consumedAt":null}}"#
        ),
    )
    .expect("write directive");
}

#[test]
fn hook_stop_injects_a_pending_directive_then_allows_the_next_turn_to_end() {
    for (agent, payload, expected_block, expected_allow) in AGENTS {
        let fixture = project_fixture(&[]);
        let root = fixture.path().to_str().unwrap().to_owned();
        write_pending_directive(fixture.path(), SESSION_ID, REASON);
        let payload = payload.replace("SESSION", SESSION_ID);
        let args = ["hook", "stop", "--agent", agent, "--root", &root];

        let blocked = run_with_stdin(&args, &payload);

        assert!(blocked.status.success(), "{}", stderr(&blocked));
        assert_eq!(stdout(&blocked).trim(), expected_block, "agent {agent}");

        // The same session stopping again must not re-inject the consumed
        // directive, which would loop the agent forever.
        let allowed = run_with_stdin(&args, &payload);

        assert!(allowed.status.success(), "{}", stderr(&allowed));
        assert_eq!(stdout(&allowed).trim(), expected_allow, "agent {agent}");
    }
}

/// A session with an exploration has to become visible to the dashboard, or the
/// reviewer has nothing to submit a verdict against — but registering it must not
/// put a directive in front of the agent, and a session with nothing to review
/// must not be registered at all.
#[test]
fn hook_stop_registers_a_session_with_an_exploration_and_no_other() {
    for (agent, payload, _, expected_allow) in AGENTS {
        let fixture = project_fixture(&[]);
        let root = fixture.path().to_str().unwrap().to_owned();
        let payload = payload.replace("SESSION", SESSION_ID);
        let args = ["hook", "stop", "--agent", agent, "--root", &root];

        // A session that asked one question about one function is not something
        // the reviewer's index should be listing.
        let unexplored = run_with_stdin(&args, &payload);

        assert!(unexplored.status.success(), "{}", stderr(&unexplored));
        assert_eq!(stdout(&unexplored).trim(), expected_allow, "agent {agent}");
        assert!(
            !directive_path(fixture.path(), SESSION_ID).exists(),
            "agent {agent} registered a session that never explored"
        );

        write_session_note(fixture.path(), "# Exploration\n");
        let explored = run_with_stdin(&args, &payload);

        assert!(explored.status.success(), "{}", stderr(&explored));
        assert_eq!(stdout(&explored).trim(), expected_allow, "agent {agent}");
        let record = fs::read_to_string(directive_path(fixture.path(), SESSION_ID))
            .expect("the session is registered for the dashboard to discover");
        assert!(
            record.contains(r#""pending": false"#),
            "agent {agent} registered the session with a directive waiting: {record}"
        );
    }
}

/// Registration is a slot, not a record of something that happened: a session
/// that has never been sent a directive must not read as one that was.
#[test]
fn registering_a_session_does_not_overwrite_a_directive_it_already_has() {
    let fixture = project_fixture(&[]);
    let root = fixture.path().to_str().unwrap().to_owned();
    // A note, so this turn boundary reaches the registration step at all.
    write_session_note(fixture.path(), "# Exploration\n");
    write_pending_directive(fixture.path(), SESSION_ID, REASON);

    let blocked = claude_stop(&root);

    assert!(blocked.status.success(), "{}", stderr(&blocked));
    assert!(
        stdout(&blocked).contains(REASON),
        "the waiting directive was registered over: {}",
        stdout(&blocked)
    );
}

#[test]
fn hook_stop_fails_loudly_on_a_payload_from_the_wrong_agent() {
    let fixture = project_fixture(&[]);
    let root = fixture.path().to_str().unwrap().to_owned();
    let (_, claude_payload, ..) = AGENTS[0];

    let output = run_with_stdin(
        &["hook", "stop", "--agent", "pi", "--root", &root],
        &claude_payload.replace("SESSION", SESSION_ID),
    );

    assert!(!output.status.success());
    assert!(output.stdout.is_empty(), "emitted a decision anyway");
    assert!(
        stderr(&output).starts_with("error: failed to parse the pi hook payload from stdin"),
        "{}",
        stderr(&output)
    );
}

/// A session scratch note holding `contents`, the state an exploration is in
/// before its change directory exists.
fn write_session_note(root: &Path, contents: &str) {
    let dir = root.join(".openspec-doc/scratch/_session");
    fs::create_dir_all(&dir).expect("create scratch dir");
    fs::write(dir.join(format!("{SESSION_ID}.md")), contents).expect("write note");
}

/// A change directory `openspec validate` accepts.
fn write_valid_change(root: &Path, name: &str) {
    let dir = root.join("openspec/changes").join(name);
    fs::create_dir_all(dir.join("specs/some-cap")).expect("create change dir");
    fs::write(
        dir.join("proposal.md"),
        "## Why\nBecause.\n\n## What Changes\n- A thing.\n",
    )
    .expect("write proposal");
    fs::write(
        dir.join("specs/some-cap/spec.md"),
        "## ADDED Requirements\n\n### Requirement: A thing\nThe system SHALL do a thing.\n\n#### Scenario: It works\n- **WHEN** asked\n- **THEN** it does the thing\n",
    )
    .expect("write spec");
    fs::write(dir.join("tasks.md"), "- [ ] 1.1 Do the thing\n").expect("write tasks");
}

/// The verdict record the dashboard appends when a reviewer submits a verdict.
fn write_verdict(root: &Path, sidecar: &str, verdict: &str, notes: &str) {
    let path = root.join(".openspec-doc/verdicts").join(sidecar);
    fs::create_dir_all(path.parent().unwrap()).expect("create verdicts dir");
    fs::write(
        &path,
        format!(
            r#"{{"id":"v1","verdict":"{verdict}","notes":"{notes}","createdAt":"2026-08-01T08:00:00Z"}}"#
        ),
    )
    .expect("write verdict");
}

/// `hook stop` for `SESSION_ID`, speaking Claude Code's wire format.
fn claude_stop(root: &str) -> Output {
    let (_, payload, ..) = AGENTS[0];
    run_with_stdin(
        &["hook", "stop", "--agent", "claude", "--root", root],
        &payload.replace("SESSION", SESSION_ID),
    )
}

/// `scratch claim` for `SESSION_ID`, the command the move-to-proposal directive
/// asks the agent to run once its change directory exists.
fn claim(root: &str, change: &str) -> Output {
    run(&[
        "scratch",
        "claim",
        "--session",
        SESSION_ID,
        "--change",
        change,
        "--root",
        root,
    ])
}

#[test]
fn hook_stop_promotes_the_scratch_note_and_reports_the_validate_outcome() {
    let fixture = project_fixture(&[]);
    let root = fixture.path().to_str().unwrap().to_owned();
    write_session_note(fixture.path(), "# Exploration\n");

    // A change appearing promotes nothing on its own; the claim is what does.
    write_valid_change(fixture.path(), "add-thing");
    let unclaimed = claude_stop(&root);
    assert!(
        !stderr(&unclaimed).contains("promoted"),
        "an unclaimed change was promoted: {}",
        stderr(&unclaimed)
    );

    assert!(claim(&root, "add-thing").status.success());
    let promoted = claude_stop(&root);

    assert!(promoted.status.success(), "{}", stderr(&promoted));
    let stderr = stderr(&promoted);
    assert!(
        stderr.contains("promoted the scratch note to add-thing"),
        "{stderr}"
    );
    assert!(
        stderr.contains("`openspec validate add-thing` passed"),
        "{stderr}"
    );
    assert!(
        fixture
            .path()
            .join(".openspec-doc/scratch/add-thing.md")
            .is_file(),
        "the note now lives at the change key"
    );
}

/// A promotion whose change does not validate still happened; the hook has to
/// say so rather than let the failure pass unremarked.
#[test]
fn hook_stop_reports_a_failing_validate_rather_than_hiding_it() {
    let fixture = project_fixture(&[]);
    let root = fixture.path().to_str().unwrap().to_owned();
    write_session_note(fixture.path(), "# Exploration\n");

    let dir = fixture.path().join("openspec/changes/add-broken");
    fs::create_dir_all(&dir).expect("create change dir");
    fs::write(dir.join("proposal.md"), "## Why\nBecause.\n").expect("write proposal");
    assert!(claim(&root, "add-broken").status.success());
    let promoted = claude_stop(&root);

    assert!(
        promoted.status.success(),
        "a failing validate must not wedge the hook"
    );
    assert!(
        stderr(&promoted).contains("`openspec validate add-broken` FAILED"),
        "{}",
        stderr(&promoted)
    );
}

#[test]
fn hook_stop_turns_a_dashboard_verdict_into_the_directive_it_injects() {
    let fixture = project_fixture(&[]);
    let root = fixture.path().to_str().unwrap().to_owned();
    write_session_note(fixture.path(), "# Exploration\n");
    write_verdict(
        fixture.path(),
        &format!("_session/{SESSION_ID}.jsonl"),
        "keep-exploring",
        "The promotion trigger is still hand-waved.",
    );

    let blocked = claude_stop(&root);

    assert!(blocked.status.success(), "{}", stderr(&blocked));
    let decision = stdout(&blocked);
    assert!(decision.contains(r#""decision":"block""#), "{decision}");
    assert!(
        decision.contains("openspec-doc dashboard"),
        "the reason says where it came from: {decision}"
    );
    assert!(
        decision.contains(&format!(".openspec-doc/scratch/_session/{SESSION_ID}.md")),
        "the reason names the note to read: {decision}"
    );
    assert!(
        !decision.contains("The promotion trigger is still hand-waved."),
        "the reviewer's notes are pointed at, not embedded: {decision}"
    );

    // The verdict is translated once: a second turn boundary must not re-inject
    // it, which would loop the agent forever.
    let allowed = claude_stop(&root);

    assert!(allowed.status.success(), "{}", stderr(&allowed));
    assert_eq!(stdout(&allowed).trim(), r#"{"continue":true}"#);
}

/// A prompt-submission payload: the common fields plus the prompt itself, none
/// of the Stop-specific ones.
const PROMPT_PAYLOAD: &str = r#"{"session_id":"SESSION","prompt_id":"p1","transcript_path":"/t.jsonl","cwd":"/x","permission_mode":"default","hook_event_name":"UserPromptSubmit","prompt":"carry on"}"#;

/// `hook prompt` for `SESSION_ID`, speaking Claude Code's wire format.
fn claude_prompt(root: &str) -> Output {
    run_with_stdin(
        &["hook", "prompt", "--agent", "claude", "--root", root],
        &PROMPT_PAYLOAD.replace("SESSION", SESSION_ID),
    )
}

/// The reviewer submits a verdict while the agent is idle, then pokes the
/// session. That poke is the turn the feedback belongs to, so it has to arrive
/// with the prompt rather than at the end of the turn the prompt starts.
#[test]
fn hook_prompt_delivers_a_standing_verdict_as_context_for_the_prompt() {
    let fixture = project_fixture(&[]);
    let root = fixture.path().to_str().unwrap().to_owned();
    write_session_note(fixture.path(), "# Exploration\n");
    write_verdict(
        fixture.path(),
        &format!("_session/{SESSION_ID}.jsonl"),
        "keep-exploring",
        "The promotion trigger is still hand-waved.",
    );

    let output = claude_prompt(&root);

    assert!(output.status.success(), "{}", stderr(&output));
    let stdout = stdout(&output);
    assert!(
        stdout.contains("openspec-doc dashboard"),
        "the context says where it came from: {stdout}"
    );
    assert!(
        stdout.contains(&format!(".openspec-doc/scratch/_session/{SESSION_ID}.md")),
        "the context names the note to read: {stdout}"
    );
    assert!(
        !stdout.contains(r#""decision""#),
        "prompt context is plain text, not a stop decision: {stdout}"
    );
}

#[test]
fn hook_prompt_delivers_a_directive_already_waiting() {
    let fixture = project_fixture(&[]);
    let root = fixture.path().to_str().unwrap().to_owned();
    write_pending_directive(fixture.path(), SESSION_ID, REASON);

    let output = claude_prompt(&root);

    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(stdout(&output).trim(), REASON);
}

#[test]
fn hook_prompt_emits_nothing_when_the_session_has_no_feedback_outstanding() {
    let fixture = project_fixture(&[]);
    let root = fixture.path().to_str().unwrap().to_owned();

    let output = claude_prompt(&root);

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(
        output.stdout.is_empty(),
        "an ordinary prompt must add no context: {}",
        stdout(&output)
    );
}

/// Promotion looks for change directories that appear as a result of agent work,
/// so the turn boundary is where it belongs. Running it here would put an
/// `openspec validate` subprocess in front of every prompt.
#[test]
fn hook_prompt_does_not_promote_the_scratch_note() {
    let fixture = project_fixture(&[]);
    let root = fixture.path().to_str().unwrap().to_owned();
    write_session_note(fixture.path(), "# Exploration\n");
    write_valid_change(fixture.path(), "add-thing");
    assert!(claim(&root, "add-thing").status.success());

    let output = claude_prompt(&root);

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(
        !stderr(&output).contains("promoted") && !stderr(&output).contains("openspec validate"),
        "the prompt hook ran the promotion check: {}",
        stderr(&output)
    );
    assert!(
        session_note_path(fixture.path()).is_file(),
        "the note was promoted at prompt time"
    );
}

/// The two delivery points share one directive. Prompt time wins the race, and
/// the turn boundary that follows must find nothing left to inject — a second
/// delivery would block a turn over feedback the agent already has.
#[test]
fn a_directive_delivered_with_a_prompt_does_not_block_the_turn_end() {
    let fixture = project_fixture(&[]);
    let root = fixture.path().to_str().unwrap().to_owned();
    write_pending_directive(fixture.path(), SESSION_ID, REASON);

    let delivered = claude_prompt(&root);
    assert_eq!(stdout(&delivered).trim(), REASON);

    let allowed = claude_stop(&root);

    assert!(allowed.status.success(), "{}", stderr(&allowed));
    assert_eq!(stdout(&allowed).trim(), r#"{"continue":true}"#);
}

#[test]
fn a_directive_delivered_at_a_turn_boundary_leaves_the_next_prompt_nothing() {
    let fixture = project_fixture(&[]);
    let root = fixture.path().to_str().unwrap().to_owned();
    write_pending_directive(fixture.path(), SESSION_ID, REASON);

    let blocked = claude_stop(&root);
    assert!(stdout(&blocked).contains(REASON), "{}", stdout(&blocked));

    let output = claude_prompt(&root);

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(
        output.stdout.is_empty(),
        "the same directive was delivered twice: {}",
        stdout(&output)
    );
}

/// A failure delivered the prompt anyway, said so on stderr, and added no
/// context. `hook prompt` sits in front of the human's own input, so a failure
/// here must cost the feedback and never the ability to type — the deliberate
/// opposite of `hook stop`, which propagates and exits non-zero.
fn assert_fails_soft(output: &Output, case: &str) {
    assert!(
        output.status.success(),
        "{case} refused the reviewer's prompt: {}",
        stderr(output)
    );
    assert!(
        output.stdout.is_empty(),
        "{case} emitted context anyway: {}",
        stdout(output)
    );
    assert!(
        stderr(output).contains("could not deliver review feedback"),
        "{case} failed silently: {}",
        stderr(output)
    );
}

#[test]
fn hook_prompt_survives_a_corrupt_directive_record() {
    let fixture = project_fixture(&[]);
    let root = fixture.path().to_str().unwrap().to_owned();
    let path = directive_path(fixture.path(), SESSION_ID);
    fs::create_dir_all(path.parent().unwrap()).expect("create directives dir");
    fs::write(&path, "{ not json").expect("write directive");

    assert_fails_soft(&claude_prompt(&root), "a corrupt directive record");
}

#[test]
fn hook_prompt_survives_a_corrupt_verdict_sidecar() {
    let fixture = project_fixture(&[]);
    let root = fixture.path().to_str().unwrap().to_owned();
    write_verdict(
        fixture.path(),
        &format!("_session/{SESSION_ID}.jsonl"),
        "keep-exploring",
        "Fine.",
    );
    fs::write(
        fixture
            .path()
            .join(".openspec-doc/verdicts/_session")
            .join(format!("{SESSION_ID}.jsonl")),
        "{ not json\n",
    )
    .expect("corrupt the verdict");

    assert_fails_soft(&claude_prompt(&root), "a corrupt verdict sidecar");
}

/// `hook stop` fails loudly on this. Here the same mistake must not be what
/// stops the reviewer from typing.
#[test]
fn hook_prompt_survives_a_payload_from_the_wrong_agent() {
    let fixture = project_fixture(&[]);
    let root = fixture.path().to_str().unwrap().to_owned();

    let output = run_with_stdin(
        &["hook", "prompt", "--agent", "pi", "--root", &root],
        &PROMPT_PAYLOAD.replace("SESSION", SESSION_ID),
    );

    assert_fails_soft(&output, "a payload from the wrong agent");
}

/// Exit zero and emit nothing, whatever went wrong and whoever reported it.
///
/// Weaker than `assert_fails_soft` on purpose: a failure that never reached the
/// command has no message of ours on stderr, only clap's.
fn assert_never_refuses(output: &Output, case: &str) {
    assert!(
        output.status.success(),
        "{case} refused the reviewer's prompt (exit {:?}): {}",
        output.status.code(),
        stderr(output)
    );
    assert!(
        output.stdout.is_empty(),
        "{case} emitted context anyway: {}",
        stdout(output)
    );
}

/// The failures that never reach `hook::prompt` at all, because argument parsing
/// and root resolution both happen before dispatch.
///
/// These matter more than the ones that do reach it. Clap exits 2 on a usage
/// error, and Claude Code treats a `UserPromptSubmit` hook exiting 2 as a block —
/// so before this was handled, one typo in a hook command string refused every
/// prompt in the session and returned clap's usage text instead of an answer.
#[test]
fn hook_prompt_never_refuses_a_prompt_over_its_own_arguments() {
    let fixture = project_fixture(&[]);
    let root = fixture.path().to_str().unwrap().to_owned();
    let payload = PROMPT_PAYLOAD.replace("SESSION", SESSION_ID);

    let cases: [(&str, Vec<&str>); 4] = [
        (
            "a typo'd flag",
            vec!["hook", "prompt", "--agnet", "claude", "--root", &root],
        ),
        ("a missing --agent", vec!["hook", "prompt", "--root", &root]),
        (
            "an unknown agent",
            vec!["hook", "prompt", "--agent", "bogus", "--root", &root],
        ),
        (
            "a root that does not resolve",
            vec![
                "hook",
                "prompt",
                "--agent",
                "claude",
                "--root",
                "/no/such/project",
            ],
        ),
    ];

    for (case, args) in cases {
        assert_never_refuses(&run_expecting_early_exit(&args, &payload), case);
    }
}

/// The fail-soft is scoped to the one command that needs it. Everything else
/// must still fail loudly, including the sibling hook.
#[test]
fn other_commands_still_exit_non_zero_on_a_usage_error() {
    for args in [
        Vec::new(),
        vec!["hook", "stop"],
        vec!["hook", "explore"],
        vec!["comment", "list"],
        vec!["no-such-command"],
    ] {
        let output = run_expecting_early_exit(&args, "");
        assert!(
            !output.status.success(),
            "{args:?} exited zero on a usage error"
        );
    }
}

/// A command-expansion payload: the common fields, none of the Stop-specific
/// ones.
const EXPANSION_PAYLOAD: &str = r#"{"session_id":"SESSION","prompt_id":"p1","transcript_path":"/t.jsonl","cwd":"/x","permission_mode":"default","hook_event_name":"UserPromptExpansion"}"#;

fn session_note_path(root: &Path) -> PathBuf {
    root.join(".openspec-doc/scratch/_session")
        .join(format!("{SESSION_ID}.md"))
}

#[test]
fn hook_explore_readies_the_note_location_and_says_where_it_is() {
    let fixture = project_fixture(&[]);
    let root = fixture.path().to_str().unwrap().to_owned();

    let output = run_with_stdin(
        &["hook", "explore", "--agent", "claude", "--root", &root],
        &EXPANSION_PAYLOAD.replace("SESSION", SESSION_ID),
    );

    assert!(output.status.success(), "{}", stderr(&output));
    let note = session_note_path(fixture.path());
    assert!(
        note.parent().expect("parent").is_dir(),
        "the note has nowhere to be written"
    );
    assert!(
        !note.exists(),
        "an empty note breaks the agent's first write: it cannot write over a file it has not read"
    );
    let stdout = stdout(&output);
    assert!(
        stdout.contains(&format!(".openspec-doc/scratch/_session/{SESSION_ID}.md")),
        "the agent is not told the resolved path: {stdout}"
    );
    assert!(stdout.contains("reviewer"), "{stdout}");
}

/// The moment the reviewer asks to be shown the exploration. Before the port
/// assignment there was no URL to give them and no way to produce one, because the
/// port was a property of a running process and nothing had started one yet.
#[test]
fn hook_explore_prints_this_projects_dashboard_url_and_starts_nothing() {
    let fixture = project_fixture(&[]);
    let root = fixture.path().to_str().unwrap().to_owned();

    let output = run_with_stdin(
        &["hook", "explore", "--agent", "claude", "--root", &root],
        &EXPANSION_PAYLOAD.replace("SESSION", SESSION_ID),
    );

    assert!(output.status.success(), "{}", stderr(&output));
    let stdout = stdout(&output);
    let url = assigned_url(&root);
    assert!(
        stdout.contains(&url),
        "the reviewer is not told where the review will be: {stdout}"
    );
    assert!(
        stdout.contains(&format!("{url}/sessions/{SESSION_ID}")),
        "the session's own page is what they want: {stdout}"
    );
    assert!(
        !directive_path(fixture.path(), SESSION_ID).exists(),
        "starting an exploration must register nothing on its own"
    );
}

/// The URL this project's dashboard belongs at, whether or not one is serving it.
fn assigned_url(root: &str) -> String {
    let output = run(&["serve", "url", "--root", root]);
    assert!(output.status.success(), "{}", stderr(&output));

    stdout(&output)
        .split_whitespace()
        .next()
        .expect("a URL on the first line")
        .to_owned()
}

/// The capability the port assignment exists to provide: an answer with nothing
/// running at all.
///
/// Also the "starts no server" assertion for every `serve` subcommand: each of
/// them asks about dashboards and none of them may leave one behind, which is
/// checked by the project still reporting nothing running afterwards.
#[test]
fn serve_url_names_the_assigned_port_with_no_dashboard_running() {
    let fixture = project_fixture(&[]);
    let root = fixture.path().to_str().unwrap().to_owned();

    let output = run(&["serve", "url", "--root", &root]);

    assert!(output.status.success(), "{}", stderr(&output));
    let stdout = stdout(&output);
    assert!(stdout.contains("http://127.0.0.1:43"), "{stdout}");
    assert!(stdout.contains("(not running)"), "{stdout}");
    // Asked twice, a project's URL is the same URL.
    assert_eq!(
        stdout,
        self::stdout(&run(&["serve", "url", "--root", &root]))
    );

    for command in [
        vec!["serve", "url", "--root", &root],
        vec!["serve", "list"],
        vec!["serve", "forget", &root],
    ] {
        let output = run(&command);
        assert!(output.status.success(), "{}", stderr(&output));
        let after = self::stdout(&run(&["serve", "url", "--root", &root]));
        assert!(
            after.contains("(not running)"),
            "`{}` left a dashboard behind: {after}",
            command.join(" ")
        );
    }
}

/// The whole point of enumeration: it answers about the machine, so it must
/// answer from a directory that is not a project at all.
#[test]
fn serve_list_works_outside_any_project_and_names_the_range_it_searched() {
    let outside = TempDir::new().expect("a directory that is not a project");

    let output = run_in(outside.path(), &["serve", "list"]);

    assert!(output.status.success(), "{}", stderr(&output));
    let stdout = stdout(&output);
    assert!(
        stdout.contains("probed ports 4321-4352"),
        "a dashboard outside the range is only explicable if the range is stated: {stdout}"
    );
}

/// Nothing running is the ordinary state of a machine between review sessions,
/// and an operator checking is getting the answer they asked for.
#[test]
fn serve_list_exits_zero_with_no_project_root_resolvable() {
    let outside = TempDir::new().expect("a directory that is not a project");

    let output = run_in(outside.path(), &["serve", "list", "--root", "/nonexistent"]);

    assert!(
        output.status.success(),
        "an unresolvable root must not stop a machine-wide question: {}",
        stderr(&output)
    );
}

#[test]
fn serve_forget_reports_an_unassigned_root_as_nothing_to_forget() {
    let fixture = project_fixture(&[]);
    let root = fixture.path().to_str().unwrap().to_owned();

    let output = run(&["serve", "forget", &root]);

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(
        stdout(&output).contains("nothing to forget"),
        "{}",
        stdout(&output)
    );
}

/// A relative or symlinked spelling of a root must forget the entry the
/// canonical one keyed, or `forget` frees a port that is still held.
#[test]
fn serve_forget_drops_the_assignment_a_non_canonical_path_names() {
    let fixture = project_fixture(&[]);
    let canonical = fixture.path().canonicalize().expect("canonicalize");
    let assigned = assigned_url(fixture.path().to_str().unwrap());
    // Names the same directory, spelled as nothing else in the registry is.
    let indirect = fixture.path().join("openspec").join("..");

    let output = run(&["serve", "forget", indirect.to_str().unwrap()]);

    assert!(output.status.success(), "{}", stderr(&output));
    let stdout = stdout(&output);
    assert!(stdout.contains("forgot"), "{stdout}");
    assert!(
        stdout.contains(&canonical.display().to_string()),
        "the canonical root is what was keyed: {stdout}"
    );
    assert!(
        !assignments().contains_key(&canonical.display().to_string()),
        "the entry survived: {assigned} still assigned"
    );
    assert!(
        self::stdout(&run(&["serve", "forget", indirect.to_str().unwrap()]))
            .contains("nothing to forget"),
        "forgetting twice claimed to forget twice"
    );
}

/// Dropping an assignment under a live dashboard hands its port to another
/// project with a server still on it.
#[test]
fn serve_forget_is_refused_while_a_dashboard_is_serving_that_root() {
    let fixture = project_fixture(&[]);
    let canonical = fixture.path().canonicalize().expect("canonicalize");
    let port = identity_server(&canonical);
    write_assignment(&canonical, port);

    let output = run(&["serve", "forget", fixture.path().to_str().unwrap()]);

    assert!(!output.status.success(), "{}", stdout(&output));
    let stderr = stderr(&output);
    assert!(stderr.contains("stop it"), "{stderr}");
    assert_eq!(
        assignments().get(&canonical.display().to_string()),
        Some(&serde_json::json!(port)),
        "a refused forget dropped the entry anyway"
    );
}

/// The error a full range produces names a way out, and this is the test that the
/// way out exists: the named command is extracted from the message and run.
#[test]
fn an_exhausted_port_range_names_a_command_the_binary_accepts() {
    let fixture = project_fixture(&[]);
    let filler = TempDir::new().expect("filler roots");
    // Every port held by a root that still exists, so nothing can be reclaimed.
    for (index, port) in (4321..=4352u16).enumerate() {
        let root = filler.path().join(format!("filler-{index}"));
        fs::create_dir_all(&root).expect("create filler root");
        write_assignment(&root, port);
    }

    let exhausted = run(&["serve", "url", "--root", fixture.path().to_str().unwrap()]);

    let message = stderr(&exhausted);
    let named = between(&message, "forget one with `", "`");
    assert!(named.starts_with("openspec-doc "), "{message}");
    let named: Vec<&str> = named.split_whitespace().skip(1).collect();
    let mut args = named.clone();
    args.push(fixture.path().to_str().unwrap());

    let accepted = run(&args);

    assert!(
        accepted.status.success(),
        "the full-range error names `{}`, which the binary does not run: {}",
        named.join(" "),
        stderr(&accepted)
    );
}

/// `Option<ServeCommand>` is what makes a bare `serve` a server rather than a
/// help screen, and further subcommands must not spend that.
#[test]
fn a_bare_serve_starts_a_server_with_every_subcommand_present() {
    let fixture = project_fixture(&[]);
    let help = run(&["serve", "--help"]);
    for command in ["url", "list", "kill", "forget"] {
        assert!(
            stdout(&help).contains(command),
            "`serve {command}` missing from:\n{}",
            stdout(&help)
        );
    }

    // A port the test itself holds, so a bare `serve` fails at the bind — which
    // is only reachable if it went to start a server instead of printing help.
    let held = std::net::TcpListener::bind("127.0.0.1:0").expect("hold a port");
    let port = held.local_addr().expect("local addr").port().to_string();
    let output = run(&[
        "serve",
        "--root",
        fixture.path().to_str().unwrap(),
        "--port",
        &port,
        "--no-open",
    ]);

    assert!(!output.status.success(), "{}", stdout(&output));
    assert!(
        !stdout(&output).contains("Usage:"),
        "a bare `serve` printed help instead of starting a server: {}",
        stdout(&output)
    );
    assert!(
        stderr(&output).contains(&port),
        "the failure is not about the port it was told to bind: {}",
        stderr(&output)
    );
}

/// The destructive reading must never be what an operator gets for typing less.
///
/// Note what is *not* tested here, deliberately: no test in this file may run
/// `serve kill --all` or a kill that matches anything. These drive the real
/// binary, so the sweep is the real port range — which is where the developer's
/// own dashboards are sitting. Killing is tested against fakes on ports the
/// operating system hands out, in `kill::tests`.
#[test]
fn serve_kill_with_no_target_stops_nothing_and_says_a_target_is_required() {
    let output = run(&["serve", "kill"]);

    assert!(!output.status.success(), "{}", stdout(&output));
    let message = stderr(&output);
    for target in ["--project", "--port", "--all"] {
        assert!(
            message.contains(target),
            "the refusal has to name the targets: {message}"
        );
    }
}

/// Nothing running for the named project is not a stop, and must not read as
/// one — the dashboard the operator meant may have been reaped by the idle
/// deadline, or may be on a port outside the range.
#[test]
fn serve_kill_reports_a_project_with_no_dashboard_as_nothing_matched() {
    let fixture = project_fixture(&[]);
    let root = fixture.path().to_str().unwrap();

    let output = run(&["serve", "kill", "--project", "--root", root]);

    assert!(
        !output.status.success(),
        "a kill that stopped nothing exited zero: {}",
        stdout(&output)
    );
    let message = stderr(&output);
    assert!(message.contains("nothing matched"), "{message}");
}

/// The port target answers about the machine, so it must answer from a
/// directory that is not a project at all.
#[test]
fn serve_kill_by_port_works_outside_any_project() {
    let outside = TempDir::new().expect("a directory that is not a project");
    // A port the operating system says is free, so it is outside the range the
    // sweep covers and cannot match anything of the developer's.
    let free = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = free.local_addr().expect("local addr").port().to_string();
    drop(free);

    let output = run_in(outside.path(), &["serve", "kill", "--port", &port]);

    assert!(!output.status.success(), "{}", stdout(&output));
    let message = stderr(&output);
    assert!(message.contains(&port), "{message}");
    assert!(
        !message.contains("openspec/config.yaml"),
        "a machine-wide kill demanded a project root: {message}"
    );
}

/// Answer identity probes with `root` for as long as the test binary runs.
///
/// The port is the operating system's, not one from the real range: the range is
/// where the developer's own dashboards are sitting, and tests here run in
/// parallel.
fn identity_server(root: &Path) -> u16 {
    use std::io::{BufRead, BufReader};
    use std::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("local addr").port();
    let body = serde_json::json!({
        "root": root.display().to_string(),
        "pid": std::process::id(),
    })
    .to_string();
    std::thread::spawn(move || {
        for connection in listener.incoming() {
            let Ok(mut stream) = connection else { break };
            let mut request = String::new();
            let _ = BufReader::new(&stream).read_line(&mut request);
            let _ = stream.write_all(
                format!(
                    "HTTP/1.0 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                    body.len()
                )
                .as_bytes(),
            );
        }
    });

    port
}

/// This test's port registry, keyed the way the binary keys it.
fn assignments() -> serde_json::Map<String, serde_json::Value> {
    let path = state_dir().join("ports.json");
    let Ok(contents) = fs::read_to_string(&path) else {
        return serde_json::Map::new();
    };
    let registry: serde_json::Value = serde_json::from_str(&contents).expect("parse the registry");

    registry["ports"]
        .as_object()
        .expect("the registry holds a port table")
        .clone()
}

/// Assign `port` to `root` in this test's registry, the way an earlier run would
/// have left it.
fn write_assignment(root: &Path, port: u16) {
    let mut ports = assignments();
    ports.insert(root.display().to_string(), port.into());
    fs::write(
        state_dir().join("ports.json"),
        serde_json::json!({ "ports": ports }).to_string(),
    )
    .expect("write the registry");
}

#[test]
fn hook_explore_leaves_an_exploration_already_written_alone() {
    let fixture = project_fixture(&[]);
    let root = fixture.path().to_str().unwrap().to_owned();
    let args = ["hook", "explore", "--agent", "claude", "--root", &root];
    let payload = EXPANSION_PAYLOAD.replace("SESSION", SESSION_ID);
    run_with_stdin(&args, &payload);
    fs::write(
        session_note_path(fixture.path()),
        "# Exploration\n\nKept.\n",
    )
    .expect("write note");

    let output = run_with_stdin(&args, &payload);

    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        fs::read_to_string(session_note_path(fixture.path())).expect("read back"),
        "# Exploration\n\nKept.\n",
        "re-entering explore mode cleared the note"
    );
}

/// Promotion reads "no note" as "nothing to promote". If an ordinary turn
/// boundary created one, every open session would promote on the first new
/// change directory and concurrent sessions would collide on the same path.
#[test]
fn hook_stop_creates_no_note_for_a_session_that_never_explored() {
    let fixture = project_fixture(&[]);
    let root = fixture.path().to_str().unwrap().to_owned();

    claude_stop(&root);

    assert!(
        !session_note_path(fixture.path()).exists(),
        "a session that never explored must not look like one that did"
    );
}

/// A claim is a statement about an exploration. Claiming with no note would
/// otherwise write a note holding nothing but the marker, and promote that.
#[test]
fn scratch_claim_without_an_exploration_fails_loudly() {
    let fixture = project_fixture(&[]);
    let root = fixture.path().to_str().unwrap().to_owned();
    write_valid_change(fixture.path(), "add-thing");

    let output = claim(&root, "add-thing");

    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("has no scratch note"),
        "{}",
        stderr(&output)
    );
    assert!(
        !session_note_path(fixture.path()).exists(),
        "a note was created from nothing"
    );
}

#[test]
fn hook_explore_fails_loudly_on_a_payload_from_the_wrong_agent() {
    let fixture = project_fixture(&[]);
    let root = fixture.path().to_str().unwrap().to_owned();

    let output = run_with_stdin(
        &["hook", "explore", "--agent", "pi", "--root", &root],
        &EXPANSION_PAYLOAD.replace("SESSION", SESSION_ID),
    );

    assert!(!output.status.success());
    assert!(
        stderr(&output).starts_with("error: failed to parse the pi hook payload from stdin"),
        "{}",
        stderr(&output)
    );
}

/// `init` end to end through the binary: the dry run writes nothing, `--yes`
/// performs exactly what the dry run printed, and a second `--yes` is a no-op.
#[test]
fn init_plans_then_performs_then_leaves_the_project_alone() {
    let fixture = project_fixture(&[".claude", ".pi"]);
    let root = fixture.path();
    fs::write(root.join("AGENTS.md"), "# House rules\n").expect("write AGENTS.md");

    let dry = run_in(root, &["init"]);
    assert!(dry.status.success(), "{}", stderr(&dry));
    let plan = stdout(&dry);
    assert!(plan.contains("harnesses: claude, pi"), "{plan}");
    assert!(plan.contains("nothing written"), "{plan}");
    assert!(!root.join(".claude/settings.json").exists());
    assert_eq!(
        fs::read_to_string(root.join("AGENTS.md")).expect("read"),
        "# House rules\n"
    );

    let performed = run_in(root, &["init", "--yes"]);
    assert!(performed.status.success(), "{}", stderr(&performed));
    let written = stdout(&performed);
    assert!(
        written.contains(&format!(
            "created   {}",
            Path::new(".claude/settings.json").display()
        )),
        "{written}"
    );
    assert!(
        written.contains(&format!(
            "created   {}",
            Path::new(".pi/extensions/openspec-doc-hook.ts").display()
        )),
        "{written}"
    );
    assert!(written.contains("modified  AGENTS.md"), "{written}");

    let settings: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join(".claude/settings.json")).expect("read settings"),
    )
    .expect("parse settings");
    for event in ["Stop", "UserPromptSubmit", "UserPromptExpansion"] {
        assert_eq!(hook_count(&settings, event), 1, "{event}: {settings}");
    }
    assert_eq!(
        settings["hooks"]["Stop"][0]["hooks"][0]["args"],
        serde_json::json!(["hook", "stop", "--agent", "claude"])
    );
    let instructions = fs::read_to_string(root.join("AGENTS.md")).expect("read AGENTS.md");
    assert!(
        instructions.starts_with("# House rules\n"),
        "{instructions}"
    );
    assert!(
        instructions.contains("<!-- openspec-doc:begin -->"),
        "{instructions}"
    );

    let again = run_in(root, &["init", "--yes"]);
    assert!(again.status.success(), "{}", stderr(&again));
    assert_eq!(stdout(&again).matches("unchanged").count(), 6);
    let settings: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join(".claude/settings.json")).expect("read settings"),
    )
    .expect("parse settings");
    for event in ["Stop", "UserPromptSubmit", "UserPromptExpansion"] {
        assert_eq!(hook_count(&settings, event), 1, "{event}: {settings}");
    }
    assert_eq!(
        fs::read_to_string(root.join("AGENTS.md")).expect("read"),
        instructions
    );
}

fn hook_count(settings: &serde_json::Value, event: &str) -> usize {
    settings["hooks"][event]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|group| group["hooks"].as_array().into_iter().flatten())
        .filter(|hook| hook["command"] == "openspec-doc")
        .count()
}

#[test]
fn init_rewrites_shell_form_entries_without_doubling() {
    let fixture = project_fixture(&[".claude"]);
    let settings = serde_json::json!({
        "hooks": {
            "Stop": [{"hooks": [{"type": "command", "command": "openspec-doc hook stop --agent claude"}]}],
            "UserPromptSubmit": [{"hooks": [{"type": "command", "command": "openspec-doc hook prompt --agent claude"}]}],
            "UserPromptExpansion": [{
                "matcher": "opsx:explore|openspec-explore",
                "hooks": [{"type": "command", "command": "openspec-doc hook explore --agent claude"}]
            }]
        }
    });
    fs::write(
        fixture.path().join(".claude/settings.json"),
        serde_json::to_string(&settings).expect("serialize settings"),
    )
    .expect("write settings");

    let output = run_in(fixture.path(), &["init", "--yes", "--skip-instructions"]);
    assert!(output.status.success(), "{}", stderr(&output));

    let settings: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(fixture.path().join(".claude/settings.json")).expect("read settings"),
    )
    .expect("parse settings");
    for event in ["Stop", "UserPromptSubmit", "UserPromptExpansion"] {
        assert_eq!(hook_count(&settings, event), 1, "{event}: {settings}");
    }
    assert!(
        !settings.to_string().contains("openspec-doc hook"),
        "{settings}"
    );
}

#[test]
fn init_without_a_harness_fails_naming_the_flag() {
    let fixture = project_fixture(&[]);

    let output = run_in(fixture.path(), &["init"]);

    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("--agent <claude|pi>"),
        "{}",
        stderr(&output)
    );
}

/// A directory holding a copied `openspec-doc` binary for putting on the
/// probes' `PATH`. The test invokes that same copy as `doctor`, so the binary
/// under test and the first binary on `PATH` remain identical without relying on
/// Unix symlink support.
fn path_with_binary() -> TempDir {
    let temp = TempDir::new().expect("temp dir");
    fs::copy(
        env!("CARGO_BIN_EXE_openspec-doc"),
        temp.path()
            .join(format!("openspec-doc{}", std::env::consts::EXE_SUFFIX)),
    )
    .expect("copy the binary under test");

    temp
}

/// `doctor` in `root`, with `bin` first on `PATH`.
fn doctor(root: &Path, bin: &Path) -> Output {
    let executable = bin.join(format!("openspec-doc{}", std::env::consts::EXE_SUFFIX));
    Command::new(executable)
        .env(STATE_DIR_ENV, state_dir())
        .current_dir(root)
        .arg("doctor")
        .env(
            "PATH",
            std::env::join_paths([bin]).expect("join the probe PATH"),
        )
        .output()
        .expect("run openspec-doc doctor")
}

/// Every file under `dir`, by relative path and contents.
fn tree(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut entries = Vec::new();
    let Ok(children) = fs::read_dir(dir) else {
        return entries;
    };

    for child in children {
        let path = child.expect("read dir entry").path();
        if path.is_dir() {
            entries.extend(tree(&path));
        } else {
            entries.push((
                path.display().to_string(),
                fs::read(&path).expect("read file"),
            ));
        }
    }

    entries.sort();
    entries
}

#[test]
fn doctor_passes_a_wired_project_and_fails_naming_a_hook_that_is_not_registered() {
    let fixture = project_fixture(&[".claude"]);
    let root = fixture.path();
    let bin = path_with_binary();
    let wired = run_in(root, &["init", "--yes", "--skip-instructions"]);
    assert!(wired.status.success(), "{}", stderr(&wired));

    let healthy = doctor(root, bin.path());

    assert!(
        healthy.status.success(),
        "{}{}",
        stdout(&healthy),
        stderr(&healthy)
    );
    let report = stdout(&healthy);
    for event in ["Stop", "UserPromptSubmit", "UserPromptExpansion"] {
        assert!(
            report.contains(&format!("pass         {event} hook")),
            "{report}"
        );
    }
    assert!(report.contains("Claude Code version"), "{report}");
    assert!(
        report.contains("`claude` was not found on PATH"),
        "{report}"
    );
    assert!(report.contains("0 failed"), "{report}");

    let settings = root.join(".claude/settings.json");
    let mut wiring: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&settings).expect("read settings"))
            .expect("parse settings");
    wiring["hooks"]
        .as_object_mut()
        .expect("hooks object")
        .remove("UserPromptSubmit");
    fs::write(&settings, wiring.to_string()).expect("write settings");

    let broken = doctor(root, bin.path());

    assert!(!broken.status.success(), "{}", stdout(&broken));
    let report = stdout(&broken);
    assert!(
        report.contains("UserPromptSubmit hook: no entry in .claude/settings.json"),
        "{report}"
    );
    assert!(report.contains("openspec-doc init"), "{report}");
    // The rest of the report is still there: one failure must not truncate it.
    assert!(report.contains("pass         Stop hook"), "{report}");
}

/// The check must not cause the failure it exists to diagnose. `hook stop`
/// consumes a pending directive, so a probe that resolved to this project would
/// silently eat the reviewer's outstanding feedback.
#[test]
fn doctor_leaves_a_pending_directive_and_the_projects_own_review_state_alone() {
    let fixture = project_fixture(&[".claude"]);
    let root = fixture.path();
    let bin = path_with_binary();
    assert!(
        run_in(root, &["init", "--yes", "--skip-instructions"])
            .status
            .success()
    );
    write_session_note(root, "# Exploration\n\nSomething to review.\n");
    write_pending_directive(root, SESSION_ID, "Address the open comments.");
    let before = tree(&root.join(".openspec-doc"));

    let output = doctor(root, bin.path());

    assert!(output.status.success(), "{}", stderr(&output));
    let directive =
        fs::read_to_string(directive_path(root, SESSION_ID)).expect("read the directive");
    assert!(
        directive.contains(r#""pending":true"#),
        "the reviewer's feedback was consumed by the check: {directive}"
    );
    assert_eq!(
        tree(&root.join(".openspec-doc")),
        before,
        "the check left something behind in the project's own state"
    );
}

#[test]
fn top_level_help_lists_every_subcommand() {
    let output = run(&["--help"]);

    assert!(output.status.success());
    let stdout = stdout(&output);
    for command in [
        "summary", "serve", "hook", "comment", "scratch", "init", "doctor",
    ] {
        assert!(
            stdout.contains(command),
            "`{command}` missing from:\n{stdout}"
        );
    }
    assert!(stdout.contains("--root <PATH>"), "{stdout}");
}

#[test]
fn subcommand_help_does_not_execute_the_subcommand() {
    for command in [
        "summary", "serve", "hook", "comment", "scratch", "init", "doctor",
    ] {
        let output = run(&[command, "--help"]);

        assert!(output.status.success(), "`{command} --help` should succeed");
        let stdout = stdout(&output);
        let binary_name = format!("openspec-doc{}", std::env::consts::EXE_SUFFIX);
        assert!(
            stdout.contains(&format!("Usage: {binary_name} {command}")),
            "{stdout}"
        );
        assert!(stdout.contains("--root <PATH>"), "{stdout}");
        assert!(stderr(&output).is_empty(), "{}", stderr(&output));
    }
}

/// The only test that starts a real dashboard, because detachment, the log, and
/// the poll that waits for an answer cannot be observed from outside the binary
/// any other way: they pass in a harness and fail in a real hook invocation.
///
/// It kills what it started using the pid that dashboard reports about itself,
/// which is the same fact `serve list` will be built on.
#[test]
fn hook_stop_starts_a_dashboard_that_outlives_it_and_serve_url_finds_it() {
    let fixture = project_fixture(&[]);
    // This one test needs the start to actually happen.
    fs::remove_dir(fixture.path().join(".openspec-doc/serve.log")).expect("unblock the serve log");
    let root = fixture.path().to_str().unwrap().to_owned();
    write_session_note(fixture.path(), "# Exploration\n\nSomething to read.\n");
    // Registered already, so this turn boundary opens no browser tab: the tab is
    // bounded by first registration, and a test run must not open one per test.
    let record = directive_path(fixture.path(), SESSION_ID);
    fs::create_dir_all(record.parent().unwrap()).expect("create directives dir");
    fs::write(
        &record,
        r#"{"pending":false,"reason":"","createdAt":"2026-08-12T08:00:00Z","consumedAt":null}"#,
    )
    .expect("register the session");

    let stopped = claude_stop(&root);
    assert!(stopped.status.success(), "{}", stderr(&stopped));

    // The hook process has exited by now, so anything still serving outlived it.
    let listed = run(&["serve", "url", "--root", &root]);
    let listed = stdout(&listed);
    let Some(url) = listed.split_whitespace().next().map(str::to_owned) else {
        panic!("no URL printed: {listed}");
    };
    let identity = identity_of(&url);
    let pid: i32 = between(&identity, "\"pid\":", "}")
        .trim()
        .parse()
        .unwrap_or_else(|error| panic!("{error} in {identity}"));

    // Killed before any assertion can fail and leak it.
    let killed = stop_dashboard(pid);

    assert!(
        listed.contains("(serving)"),
        "the started dashboard was not found: {listed}\nhook said: {}",
        stderr(&stopped)
    );
    assert!(
        identity.contains(&format!(
            "\"root\":\"{}\"",
            fixture.path().canonicalize().unwrap().display()
        )),
        "the dashboard serves someone else: {identity}"
    );
    assert!(
        fixture.path().join(".openspec-doc/serve.log").is_file(),
        "the dashboard's output has nowhere to explain a failed start"
    );
    assert!(
        killed.status.success(),
        "could not stop the dashboard this test started"
    );
}

/// Stop a dashboard process using the platform's process utility.
fn stop_dashboard(pid: i32) -> Output {
    #[cfg(unix)]
    let mut command = Command::new("kill");
    #[cfg(unix)]
    let args = [pid.to_string()];

    #[cfg(windows)]
    let mut command = Command::new("taskkill");
    #[cfg(windows)]
    let args = [
        "/PID".to_owned(),
        pid.to_string(),
        "/T".to_owned(),
        "/F".to_owned(),
    ];

    command.args(args).output().expect("stop the dashboard")
}

/// One fixed `GET` to the identity route, the way the hook's own probe does it.
fn identity_of(url: &str) -> String {
    use std::io::Read;
    use std::net::TcpStream;

    let address = url.trim_start_matches("http://");
    let mut stream = TcpStream::connect(address).expect("connect to the dashboard");
    stream
        .write_all(
            format!("GET /api/identity HTTP/1.0\r\nHost: {address}\r\nConnection: close\r\n\r\n")
                .as_bytes(),
        )
        .expect("write request");
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("read response");
    response
}

fn between<'a>(haystack: &'a str, after: &str, before: &str) -> &'a str {
    let rest = haystack
        .split_once(after)
        .unwrap_or_else(|| panic!("no {after:?} in {haystack}"))
        .1;
    rest.split_once(before)
        .unwrap_or_else(|| panic!("no {before:?} after {after:?} in {haystack}"))
        .0
}

/// A change with the artifacts an approval covers, and no review history.
fn approvable_project() -> TempDir {
    let fixture = project_fixture(&["openspec/changes/add-thing"]);
    let dir = fixture.path().join("openspec/changes/add-thing");
    fs::write(dir.join("proposal.md"), "## Why\n\nBecause.\n").expect("write proposal");
    fs::write(dir.join("tasks.md"), "- [ ] 1.1 Do it\n").expect("write tasks");
    fixture
}

/// The precheck's own contract. Every caller reads the status code, so the one
/// state that exits zero is the one that clears the work.
#[test]
fn approval_state_exits_zero_only_while_the_change_is_approved() {
    let fixture = approvable_project();
    let root = fixture.path().to_str().expect("utf-8 root");
    let args = ["approval", "state", "--root", root, "--change", "add-thing"];

    let before = run(&args);
    assert!(
        !before.status.success(),
        "an unapproved change must not read as clearance: {}",
        stdout(&before)
    );
    assert!(
        stdout(&before).contains("add-thing is not approved"),
        "{}",
        stdout(&before)
    );
    assert!(
        stdout(&before).contains("no approval"),
        "the reason belongs beside the state: {}",
        stdout(&before)
    );

    // Recorded the way the dashboard records it, since the bulk act and the
    // approval control are browser-side by design.
    let project = openspec_doc_core::project_at(fixture.path()).expect("project");
    openspec_doc_core::approval::submit(&project, "add-thing").expect("approve");

    let approved = run(&args);
    assert!(approved.status.success(), "{}", stderr(&approved));
    assert!(
        stdout(&approved).contains("add-thing is approved"),
        "{}",
        stdout(&approved)
    );

    fs::write(
        fixture
            .path()
            .join("openspec/changes/add-thing/proposal.md"),
        "## Why\n\nA different reason.\n",
    )
    .expect("edit proposal");

    let stale = run(&args);
    assert!(
        !stale.status.success(),
        "a stale approval must not read as clearance: {}",
        stdout(&stale)
    );
    assert!(
        stdout(&stale).contains("add-thing is stale"),
        "{}",
        stdout(&stale)
    );
    assert!(
        stdout(&stale).contains("proposal.md"),
        "a stale state has to name what moved: {}",
        stdout(&stale)
    );
}

/// Ticking a checkbox rewrites `tasks.md` on essentially every turn of
/// implementation. A precheck that failed on that would be ignored within
/// minutes of anyone using it.
#[test]
fn approval_state_stays_approved_when_only_tasks_change() {
    let fixture = approvable_project();
    let root = fixture.path().to_str().expect("utf-8 root");
    let project = openspec_doc_core::project_at(fixture.path()).expect("project");
    openspec_doc_core::approval::submit(&project, "add-thing").expect("approve");

    fs::write(
        fixture.path().join("openspec/changes/add-thing/tasks.md"),
        "- [x] 1.1 Do it\n",
    )
    .expect("tick the task");

    let output = run(&["approval", "state", "--root", root, "--change", "add-thing"]);

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(
        stdout(&output).contains("add-thing is approved"),
        "{}",
        stdout(&output)
    );
}

/// A precheck that answered "not approved" for a name nobody ever created would
/// report a typo as a review problem.
#[test]
fn approval_state_fails_loudly_on_an_unknown_change() {
    let fixture = approvable_project();
    let root = fixture.path().to_str().expect("utf-8 root");

    let output = run(&[
        "approval",
        "state",
        "--root",
        root,
        "--change",
        "add-nothing",
    ]);

    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("no active change named add-nothing"),
        "{}",
        stderr(&output)
    );
    assert!(
        !stdout(&output).contains("not approved"),
        "an unknown change must not be reported as an unapproved one: {}",
        stdout(&output)
    );
}

/// Not shipping the affordance is not the same as it being impossible — the
/// verdict sidecar is a file — but a single command that cleared an agent's own
/// feedback and approved its own change is one the agent would have.
#[test]
fn no_command_line_surface_resolves_comments_in_bulk() {
    let help = stdout(&run(&["--help"]));
    assert!(help.contains("approval"), "{help}");

    let approval = stdout(&run(&["approval", "--help"]));
    let comment = stdout(&run(&["comment", "--help"]));

    for surface in [&approval, &comment] {
        assert!(
            !surface.contains("resolve-all") && !surface.contains("--all"),
            "the bulk act is browser-side only: {surface}"
        );
    }
}
