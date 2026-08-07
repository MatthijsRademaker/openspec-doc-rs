import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { join, resolve } from "node:path";

const repoRoot = resolve(fileURLToPath(new URL(".", import.meta.url)), "..");
const sharedSkillPaths = [
	"frontend-design/SKILL.md",
	"openspec-visual-language/SKILL.md",
	"shadcn-vue/SKILL.md",
	"openspec-doc-dashboard/SKILL.md",
	"browser-verification/SKILL.md",
];

const drift = [];
for (const relativePath of sharedSkillPaths) {
	const piPath = join(repoRoot, ".pi", "skills", relativePath);
	const claudePath = join(repoRoot, ".claude", "skills", relativePath);
	let piContents;
	let claudeContents;
	try {
		[piContents, claudeContents] = await Promise.all([
			readFile(piPath),
			readFile(claudePath),
		]);
	} catch (error) {
		throw new Error(`Unable to read mirrored skill ${relativePath}`, {
			cause: error,
		});
	}
	if (!piContents.equals(claudeContents)) {
		drift.push(relativePath);
	}
}

if (drift.length > 0) {
	for (const relativePath of drift) {
		process.stderr.write(`harness resource drift: ${relativePath}\n`);
	}
	process.exitCode = 1;
} else {
	process.stdout.write(
		`harness resources match: ${sharedSkillPaths.length} Pi/Claude skill files\n`,
	);
}
