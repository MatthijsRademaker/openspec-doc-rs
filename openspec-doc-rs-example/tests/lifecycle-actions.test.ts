import { mkdtemp, readFile, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { parseTaskChecklist } from "../src/core/task-checklist.js";
import { startViewerServer } from "../src/server/dev-server.js";
import { copyFixture } from "./helpers.js";

interface ActionTestPayload {
  error?: string;
  result: {
    ok?: boolean;
    output?: string;
    command?: string[];
    summary?: string;
    refreshed?: boolean;
  };
}

describe("task checklist parsing", () => {
  it("returns checklist line indexes and checked states", () => {
    expect(parseTaskChecklist("## Tasks\n\n- [x] First\n- [ ] Second\nnot a task")).toEqual([
      { line: 3, checked: true, text: "First" },
      { line: 4, checked: false, text: "Second" },
    ]);
  });
});

describe("browser lifecycle action API", () => {
  it("rejects unknown lifecycle actions without executing arbitrary command data", async () => {
    const root = await createFixtureCopy("changes-openspec");
    const server = await startViewerServer({ root, port: 0 });
    try {
      const response = await postAction(server.url, {
        kind: "shell",
        command: "touch SHOULD_NOT_EXIST",
      });
      const payload = await readActionPayload(response);

      expect(response.status).toBe(400);
      expect(payload.error).toContain("Unknown lifecycle action");
    } finally {
      await server.close();
      await rm(root, { recursive: true, force: true });
    }
  });

  it("returns handoff guidance without mutating tasks", async () => {
    const root = await createFixtureCopy("changes-openspec");
    const tasksPath = path.join(root, "openspec/changes/add-core/tasks.md");
    const before = await readFile(tasksPath, "utf8");
    const server = await startViewerServer({ root, port: 0 });
    try {
      const response = await postAction(server.url, {
        kind: "apply-handoff",
        changeName: "add-core",
      });
      const payload = await readActionPayload(response);

      expect(response.status).toBe(200);
      expect(payload.result.ok).toBe(true);
      expect(payload.result.output).toContain("/opsx-apply add-core");
      expect(await readFile(tasksPath, "utf8")).toBe(before);
    } finally {
      await server.close();
      await rm(root, { recursive: true, force: true });
    }
  });

  it("runs validation through the allowlisted command endpoint", async () => {
    const root = await createFixtureCopy("changes-openspec");
    const server = await startViewerServer({ root, port: 0 });
    try {
      const response = await postAction(server.url, {
        kind: "validate",
        changeName: "add-core",
        command: "echo ignored",
      });
      const payload = await readActionPayload(response);

      expect(response.status).toBe(200);
      expect(payload.result.command).toEqual(["openspec", "validate", "add-core"]);
      expect(payload.result.summary).toContain("openspec validate add-core");
    } finally {
      await server.close();
      await rm(root, { recursive: true, force: true });
    }
  });

  it("toggles only the requested task checklist line and rejects stale task lines", async () => {
    const root = await createFixtureCopy("changes-openspec");
    const tasksPath = path.join(root, "openspec/changes/add-core/tasks.md");
    const server = await startViewerServer({ root, port: 0 });
    try {
      const response = await postAction(server.url, {
        kind: "mark-task",
        changeName: "add-core",
        line: 4,
        checked: true,
      });
      const payload = await readActionPayload(response);
      const updatedTasks = await readFile(tasksPath, "utf8");

      expect(response.status).toBe(200);
      expect(payload.result.refreshed).toBe(true);
      expect(updatedTasks).toContain("- [x] First task");
      expect(updatedTasks).toContain("- [x] Second task");

      const stale = await postAction(server.url, {
        kind: "mark-task",
        changeName: "add-core",
        line: 1,
        checked: true,
      });
      const stalePayload = await readActionPayload(stale);
      expect(stale.status).toBe(409);
      expect(stalePayload.error).toContain("not a checklist item");
    } finally {
      await server.close();
      await rm(root, { recursive: true, force: true });
    }
  });

  it("archives active changes through the allowlisted archive command and refreshes the model", async () => {
    const root = await createFixtureCopy("changes-openspec");
    const server = await startViewerServer({ root, port: 0 });
    try {
      const response = await postAction(server.url, {
        kind: "archive",
        changeName: "add-core",
      });
      const payload = await readActionPayload(response);
      const changesPage = await fetch(`${server.url}/changes`);
      const archivePage = await fetch(`${server.url}/archive`);

      expect(response.status).toBe(200);
      expect(payload.result.command).toEqual(["openspec", "archive", "add-core", "--yes"]);
      expect(payload.result.refreshed).toBe(true);
      expect(await changesPage.text()).not.toContain("add-core");
      expect(await archivePage.text()).toContain("add-core");
    } finally {
      await server.close();
      await rm(root, { recursive: true, force: true });
    }
  });
});

async function createFixtureCopy(name: string): Promise<string> {
  const root = await mkdtemp(path.join(os.tmpdir(), "openspec-doc-actions-"));
  await copyFixture(name, root);
  return root;
}

async function readActionPayload(response: Response): Promise<ActionTestPayload> {
  return (await response.json()) as ActionTestPayload;
}

async function postAction(url: string, body: unknown): Promise<Response> {
  return fetch(`${url}/api/lifecycle-actions`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
  });
}
