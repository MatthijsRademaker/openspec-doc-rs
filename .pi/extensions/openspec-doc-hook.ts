/**
 * openspec-doc hook bridge for pi.dev.
 *
 * pi has no external-process hook, so this extension is pi's half of the
 * bridge: at each turn boundary it hands the session's identity to
 * `openspec-doc hook stop --agent pi` and, when a pending directive comes back,
 * re-injects that directive's reason as a follow-up message — keeping the
 * session alive exactly as Claude Code's Stop hook does.
 *
 * All decision logic lives in the Rust binary. This file only translates
 * between pi's extension API and that binary's stdin/stdout.
 *
 * Set OPENSPEC_DOC_BIN to run a binary that is not on PATH (e.g. this repo's
 * own `target/debug/openspec-doc` during verification).
 *
 * VERIFICATION STATUS — proven end to end in a real interactive pi session:
 * the directive was found at the turn boundary, injected as a user message, and
 * the agent started a fresh turn acting on it. A real captured payload from this
 * extension is the PI_STOP fixture in crates/core/src/hook/adapter.rs.
 *
 * It is also proven from a single scripted `pi -p` run, against a verdict rather
 * than a hand-written directive: the agent seeds a scratch note and a
 * keep-exploring verdict for its own session via $PI_SESSION_ID, and at
 * `agent_end` the hook translates that verdict, the directive comes back, and the
 * agent answers the reviewer's question instead of the prompt it was given. No
 * session resume is involved — the injection happens in-process through
 * `sendUserMessage`, so one invocation covers the whole loop.
 *
 * Two pi quirks make that harness fiddlier than it looks. Both are tied to the
 * session flags, so the recipe above (neither flag, agent seeds its own state)
 * sidesteps them; interactive use is unaffected either way. They will still cost
 * anyone who reaches for the obvious approach a lot of time:
 *   1. `--session-id` for a session that does not exist yet, and `--continue`,
 *      both perform a session *replacement*. A handler that then touches the
 *      captured `pi`/`ctx` throws "extension ctx is stale after session
 *      replacement" — this hit an unrelated third-party extension (pi-lens) in
 *      the same run, so it is not specific to this file. It is also why the
 *      handler below sits on `agent_end` rather than `agent_settled`.
 *   2. An `agent_end` handler combined with `--session-id` hangs pi before any
 *      output, handler never firing. Without `--session-id` it works. Cause
 *      unknown.
 * So: launch pi with neither flag and have the agent write its own session's
 * state via $PI_SESSION_ID from the bash tool.
 */
import { spawn } from "node:child_process";
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";

/** What `hook stop --agent pi` prints on stdout. */
interface Decision {
	action: "continue" | "stop";
	message?: string;
}

interface HookResult {
	stdout: string;
	stderr: string;
	code: number;
}

/**
 * Run the hook binary with `payload` on stdin. `pi.exec()` cannot pipe stdin
 * (its ExecOptions has no such field), so spawn directly.
 */
function runHook(cwd: string, payload: string): Promise<HookResult> {
	const bin = process.env.OPENSPEC_DOC_BIN ?? "openspec-doc";

	return new Promise((resolve, reject) => {
		const child = spawn(bin, ["hook", "stop", "--agent", "pi"], { cwd });
		let stdout = "";
		let stderr = "";

		child.stdout.on("data", (chunk) => {
			stdout += chunk;
		});
		child.stderr.on("data", (chunk) => {
			stderr += chunk;
		});
		child.on("error", reject);
		child.on("close", (code) => resolve({ stdout, stderr, code: code ?? -1 }));

		child.stdin.end(payload);
	});
}

export default function (pi: ExtensionAPI) {
	// `agent_end`, not `agent_settled`: by the time `agent_settled` fires the
	// session has already been replaced in non-interactive (`-p`) runs, and
	// `sendUserMessage` then throws on a stale ctx. `deliverAs: "followUp"`
	// already holds the injection until the agent has no tool calls left, so
	// firing at `agent_end` loses nothing.
	pi.on("agent_end", async (_event, ctx) => {
		const transcriptPath = ctx.sessionManager.getSessionFile();
		// An ephemeral session (`--no-session`) has no transcript and no stable
		// identity to key a directive by, so there is nothing to look up.
		if (!transcriptPath) return;

		const payload = JSON.stringify({
			sessionId: ctx.sessionManager.getSessionId(),
			transcriptPath,
			cwd: ctx.cwd,
		});

		let result: HookResult;
		try {
			result = await runHook(ctx.cwd, payload);
		} catch (error) {
			ctx.ui.notify(`openspec-doc hook stop could not run: ${error}`, "warning");
			return;
		}

		// A non-zero exit is a broken hook, not a directive: surface it and let
		// the turn end rather than guessing at a decision.
		if (result.code !== 0) {
			ctx.ui.notify(`openspec-doc hook stop failed: ${result.stderr.trim()}`, "warning");
			return;
		}

		const decision: Decision = JSON.parse(result.stdout);
		if (decision.action !== "continue" || !decision.message) return;

		pi.sendUserMessage(decision.message, { deliverAs: "followUp" });
	});
}
