/**
 * openspec-doc hook bridge for pi.dev.
 *
 * pi has no external-process hook, so this extension is pi's half of the
 * bridge, at all three of the delivery points:
 *
 *   - `before_agent_start` → `openspec-doc hook prompt --agent pi`, which puts a
 *     standing verdict in context at the *start* of the turn the prompt begins,
 *     where Claude Code uses `UserPromptSubmit`.
 *   - `agent_end` → `openspec-doc hook stop --agent pi`, which re-injects a
 *     pending directive as a follow-up message, keeping the session alive
 *     exactly as Claude Code's Stop hook does.
 *   - `input` → `openspec-doc hook explore --agent pi`, which readies the
 *     session's scratch note when the submitted input starts an exploration,
 *     where Claude Code uses `UserPromptExpansion`.
 *
 * A directive is consumed once across the two: whichever fires first delivers
 * it, and that is enforced by the binary, not here.
 *
 * All decision logic lives in the Rust binary. This file only translates
 * between pi's extension API and that binary's stdin/stdout.
 *
 * Set OPENSPEC_DOC_BIN to run a binary that is not on PATH (e.g. this repo's
 * own `target/debug/openspec-doc` during verification).
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
 * The two ways an exploration is started on pi: the prompt template
 * `.pi/prompts/opsx-explore.md` and the `openspec-explore` skill. Both names are
 * this repository's own, and both appear verbatim in raw input.
 */
const EXPLORE_COMMAND = /^\s*\/(?:opsx-explore|skill:openspec-explore)(?:\s|$)/;

/**
 * Run the hook binary with `payload` on stdin. `pi.exec()` cannot pipe stdin
 * (its ExecOptions has no such field), so spawn directly.
 */
function runHook(
	cwd: string,
	subcommand: "stop" | "prompt" | "explore",
	payload: string,
): Promise<HookResult> {
	const bin = process.env.OPENSPEC_DOC_BIN ?? "openspec-doc";

	return new Promise((resolve, reject) => {
		const child = spawn(bin, ["hook", subcommand, "--agent", "pi"], { cwd });
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
	// pi's half of prompt-time delivery. `before_agent_start` fires after the
	// prompt is submitted and before the agent loop, which is where Claude Code's
	// `UserPromptSubmit` sits, so a standing verdict reaches the model at the
	// start of the turn it belongs to rather than at the end of it.
	//
	// `hook prompt` exits zero on every failure by design — it sits in front of
	// the human's own input, where refusing the prompt is worse than delivering
	// nothing — so there is no non-zero branch to handle here. An empty stdout is
	// the ordinary case: nothing is outstanding for this session.
	pi.on("before_agent_start", async (_event, ctx) => {
		if (!ctx.sessionManager.getSessionFile()) return;

		const payload = JSON.stringify({ sessionId: ctx.sessionManager.getSessionId() });

		let result: HookResult;
		try {
			result = await runHook(ctx.cwd, "prompt", payload);
		} catch (error) {
			ctx.ui.notify(`openspec-doc hook prompt could not run: ${error}`, "warning");
			return;
		}

		const reason = result.stdout.trim();
		if (!reason) return;

		return {
			message: {
				customType: "openspec-doc-directive",
				content: reason,
				display: true,
			},
		};
	});

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
			result = await runHook(ctx.cwd, "stop", payload);
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

	// pi's half of explore detection. Claude Code dispatches this on
	// `UserPromptExpansion`, whose matcher runs against the bare command name; pi
	// has no such event, so the match is on raw input and `input` is the only
	// event that sees it. `input` fires before skill and prompt-template
	// expansion, which is what makes that possible and is also why the match is
	// not made later: by `before_agent_start` the command is gone, replaced by the
	// expanded body of `.pi/prompts/opsx-explore.md` or a `<skill>` block. That
	// prose is upstream's to rewrite, so matching it would stop detection silently
	// — the exact failure this capability is specified against.
	//
	// This is deliberately not a `pi.registerCommand("opsx-explore")`. pi
	// dispatches extension commands *before* the input event, so registering the
	// name would shadow `.pi/prompts/opsx-explore.md` and the exploration
	// instructions would stop being delivered while the note started appearing.
	//
	// Anchoring to the leading token, rather than searching the message, keeps a
	// question *about* the command from starting an exploration.
	pi.on("input", async (event, ctx) => {
		if (!EXPLORE_COMMAND.test(event.text)) return;
		if (!ctx.sessionManager.getSessionFile()) return;

		const payload = JSON.stringify({ sessionId: ctx.sessionManager.getSessionId() });

		let result: HookResult;
		try {
			result = await runHook(ctx.cwd, "explore", payload);
		} catch (error) {
			ctx.ui.notify(`openspec-doc hook explore could not run: ${error}`, "warning");
			return;
		}

		if (result.code !== 0) {
			ctx.ui.notify(`openspec-doc hook explore failed: ${result.stderr.trim()}`, "warning");
			return;
		}

		const instruction = result.stdout.trim();
		if (!instruction) return;

		// `before_agent_start` returns its directive as a message on the event
		// result; an `input` handler's result can only continue, transform or
		// swallow the input, so the equivalent injection is `deliverAs: "nextTurn"`
		// — pi appends those alongside the user message of the turn this input is
		// about to start. The input itself is never transformed.
		pi.sendMessage(
			{ customType: "openspec-doc-directive", content: instruction, display: true },
			{ deliverAs: "nextTurn" },
		);
	});
}
