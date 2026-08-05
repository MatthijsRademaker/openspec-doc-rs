import * as path from "node:path";
import { defineConfig } from "@rspress/core";
import { pluginLlms } from "@rspress/plugin-llms";

/**
 * A section of the docs, as it appears in `llms.txt`.
 *
 * The taxonomy is written out by hand rather than derived from the directory
 * tree. A generated index reflects how files happen to be arranged; this one
 * says what an agent should read and why, which is the only useful thing an
 * index can do for a reader with a budget.
 */
interface PageEntry {
	title: string;
	path: string;
	desc: string;
}

interface Section {
	title: string;
	highlight: string;
	pages: PageEntry[];
}

const TAXONOMY: Section[] = [
	{
		title: "Start here",
		highlight:
			"What the tool is for, the scope it is being built to, and how to get the review loop running.",
		pages: [
			{
				title: "Overview",
				path: "/index.md",
				desc: "What openspec-doc is: a local review dashboard for OpenSpec projects plus an agent hook bridge",
			},
			{
				title: "Vision & MVP scope",
				path: "/vision.md",
				desc: "The problem being solved, what is in the MVP, what is deliberately excluded, and how completion is judged",
			},
			{
				title: "Roadmap",
				path: "/roadmap.md",
				desc: "Shipped capabilities and the open changes, each triaged against MVP scope",
			},
			{
				title: "Quickstart",
				path: "/quickstart.md",
				desc: "Install, wire the agent hooks, start the dashboard, and drive one full review loop",
			},
		],
	},
	{
		title: "Concepts",
		highlight:
			"The four ideas the codebase is built on. Read these before changing anything: each one is load-bearing and each was arrived at by getting it wrong first.",
		pages: [
			{
				title: "The review loop",
				path: "/concepts/review-loop.md",
				desc: "How an exploration becomes a note, a verdict becomes a directive, and a directive reaches the agent at a turn boundary",
			},
			{
				title: "Scoping: session and change keys",
				path: "/concepts/scoping.md",
				desc: "Why every route and sidecar is keyed by either session id or change name, and what promotion does to that key",
			},
			{
				title: "Anchoring",
				path: "/concepts/anchoring.md",
				desc: "How a comment stays attached to a span of markdown across edits, and how it reports the drift when it cannot",
			},
			{
				title: "Pointer, not embed",
				path: "/concepts/pointer-not-embed.md",
				desc: "Why an injected directive names files instead of quoting the reviewer, and why that is what stops an agent refusing it",
			},
		],
	},
	{
		title: "Reference",
		highlight:
			"The exact surfaces: commands, on-disk file formats, HTTP routes, and agent hook wiring.",
		pages: [
			{
				title: "CLI",
				path: "/reference/cli.md",
				desc: "Every openspec-doc subcommand with its flags and output",
			},
			{
				title: "On-disk state",
				path: "/reference/on-disk-state.md",
				desc: "Every file the tool writes under .openspec-doc/, its format, and the rules that govern it",
			},
			{
				title: "HTTP routes",
				path: "/reference/routes.md",
				desc: "The dashboard's route table, form payloads, and verdict scoping rules",
			},
			{
				title: "Agent hooks",
				path: "/reference/hooks.md",
				desc: "Wiring the Stop and explore hooks for Claude Code and pi.dev, including the matcher values that work",
			},
		],
	},
	{
		title: "Development",
		highlight:
			"Working on openspec-doc itself: layout, tests, conventions, and the manual checks no test can replace.",
		pages: [
			{
				title: "Testing",
				path: "/development/testing.md",
				desc: "Test layout, what is hermetic, the known-failing watcher tests, and what automated tests cannot establish here",
			},
			{
				title: "Manual verification",
				path: "/development/manual-verification.md",
				desc: "A copy-pasteable throwaway project and a walkthrough of every review interaction, including the browser-only checks",
			},
			{
				title: "Conventions",
				path: "/development/conventions.md",
				desc: "Crate layout and the working rules that show up most in the code",
			},
		],
	},
];

function sectionSlug(title: string): string {
	return title
		.toLowerCase()
		.replace(/[^a-z0-9]+/g, "-")
		.replace(/^-+|-+$/g, "");
}

/**
 * The root `llms.txt`: one entry per section, pointing at a per-section file.
 *
 * Split this way so an agent can load the map cheaply and then pull only the
 * section it needs, rather than paying for the whole tree to answer one
 * question.
 */
function rootIndex(): string {
	const lines = [
		"# openspec-doc",
		"",
		"> A local review dashboard for OpenSpec projects, plus an agent hook bridge.",
		"> A coding agent's exploration is readable in a browser while it happens; the",
		"> reviewer comments and submits a phase verdict, and that verdict reaches the",
		"> agent as a directive at its next turn boundary.",
		"",
	];

	for (const section of TAXONOMY) {
		lines.push(`## ${section.title}`);
		lines.push(section.highlight);
		lines.push(`-> llms-${sectionSlug(section.title)}.txt`);
		lines.push("");
	}

	return lines.join("\n");
}

function sectionIndex(section: Section): string {
	const lines = [`# ${section.title}`, section.highlight, ""];

	for (const page of section.pages) {
		lines.push(`- [${page.title}](${page.path}): ${page.desc}`);
	}
	lines.push("");

	return lines.join("\n");
}

function writeSectionFiles(): void {
	// Written straight to the build output: the plugin owns llms.txt itself, and
	// these siblings are ours.
	const fs = require("node:fs");
	const outDir = path.join(__dirname, "doc_build");
	if (!fs.existsSync(outDir)) fs.mkdirSync(outDir, { recursive: true });

	for (const section of TAXONOMY) {
		fs.writeFileSync(
			path.join(outDir, `llms-${sectionSlug(section.title)}.txt`),
			sectionIndex(section),
			"utf-8",
		);
	}
}

export default defineConfig({
	root: path.join(__dirname, "docs"),
	outDir: path.join(__dirname, "doc_build"),
	title: "openspec-doc",
	description:
		"A local review dashboard for OpenSpec projects, plus an agent hook bridge",
	plugins: [
		pluginLlms({
			llmsTxt: {
				name: "llms.txt",
				onAfterLlmsTxtGenerate: () => {
					writeSectionFiles();
					return rootIndex();
				},
			},
			exclude: ({ page }) => page.routePath === "/",
		}),
	],
	themeConfig: {
		socialLinks: [
			{
				icon: "github",
				mode: "link",
				content: "https://github.com/matthijsrademaker/openspec-doc-rs",
			},
		],
	},
});
