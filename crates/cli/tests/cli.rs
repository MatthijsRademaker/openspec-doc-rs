//! End-to-end tests driving the built `openspec-doc` binary.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tempfile::TempDir;

/// A temp directory that is a valid OpenSpec project root, containing each of
/// `dirs` (relative paths).
fn project_fixture(dirs: &[&str]) -> TempDir {
    let temp = TempDir::new().expect("temp dir");
    for dir in dirs {
        fs::create_dir_all(temp.path().join(dir)).expect("create fixture dir");
    }
    fs::create_dir_all(temp.path().join("openspec")).expect("create openspec dir");
    fs::write(temp.path().join("openspec/config.yaml"), "").expect("write config");
    temp
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_openspec-doc"))
        .args(args)
        .output()
        .expect("run openspec-doc")
}

fn run_in(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_openspec-doc"))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("run openspec-doc")
}

fn run_with_stdin(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_openspec-doc"))
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
    let mut child = Command::new(env!("CARGO_BIN_EXE_openspec-doc"))
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

/// The session still has to become visible to the dashboard, or the reviewer
/// has nothing to submit a verdict against — but registering it must not put a
/// directive in front of the agent.
#[test]
fn hook_stop_allows_and_registers_the_session_when_it_has_no_directive() {
    for (agent, payload, _, expected_allow) in AGENTS {
        let fixture = project_fixture(&[]);
        let root = fixture.path().to_str().unwrap().to_owned();
        let payload = payload.replace("SESSION", SESSION_ID);

        let output = run_with_stdin(
            &["hook", "stop", "--agent", agent, "--root", &root],
            &payload,
        );

        assert!(output.status.success(), "{}", stderr(&output));
        assert_eq!(stdout(&output).trim(), expected_allow, "agent {agent}");
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

#[test]
fn top_level_help_lists_every_subcommand() {
    let output = run(&["--help"]);

    assert!(output.status.success());
    let stdout = stdout(&output);
    for command in ["summary", "serve", "hook", "comment", "scratch"] {
        assert!(
            stdout.contains(command),
            "`{command}` missing from:\n{stdout}"
        );
    }
    assert!(stdout.contains("--root <PATH>"), "{stdout}");
}

#[test]
fn subcommand_help_does_not_execute_the_subcommand() {
    for command in ["summary", "serve", "hook", "comment", "scratch"] {
        let output = run(&[command, "--help"]);

        assert!(output.status.success(), "`{command} --help` should succeed");
        let stdout = stdout(&output);
        assert!(
            stdout.contains(&format!("Usage: openspec-doc {command}")),
            "{stdout}"
        );
        assert!(stdout.contains("--root <PATH>"), "{stdout}");
        assert!(stderr(&output).is_empty(), "{}", stderr(&output));
    }
}
