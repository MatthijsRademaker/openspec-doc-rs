import { existsSync } from "node:fs";
import { mkdtemp, readFile, readdir, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { createReviewAgentBundle } from "../src/core/review-agent-bundle.js";
import { analyzeProject } from "../src/core/analyzer.js";
import { parseProject } from "../src/core/parser.js";
import { scanProject } from "../src/core/scanner.js";
import { startViewerServer } from "../src/server/dev-server.js";
import type {
  BrowserReviewAgentEvent,
  PiReviewBridge,
  PiReviewSessionHandle,
} from "../src/server/pi-bridge.js";
import {
  appendReviewAgentResponseCompleted,
  createBrowserReviewAgentSession,
  listRecentReviewAgentSessions,
  validateReviewAgentSessionRequest,
} from "../src/server/review-agent.js";
import { buildViewerModelFromProject } from "../src/viewer/model.js";
import { renderRoute } from "../src/viewer/render.js";
import { copyFixture, fixturePath } from "./helpers.js";

async function buildFixtureModel(name: string, reviewCompanionEnabled = false) {
  const scan = await scanProject(fixturePath(name));
  if (!scan.ok) {
    throw new Error(scan.fatalError.message);
  }

  const parsed = await parseProject(scan.project);
  const analysis = analyzeProject(parsed);
  return {
    parsed,
    analysis,
    model: await buildViewerModelFromProject(parsed, analysis, { reviewCompanionEnabled }),
  };
}

describe("viewer model", () => {
  it("uses the same scan, parse, and warning results as the core summary", async () => {
    const { analysis, model } = await buildFixtureModel("changes-openspec");

    expect(model.summary).toEqual(analysis.summary);
    expect(model.specs.map((spec) => spec.relativePath)).toEqual(["openspec/specs/core/spec.md"]);
    expect(model.activeChanges.map((change) => change.name)).toEqual(["add-core"]);
    expect(model.archivedChanges.map((change) => change.name)).toEqual(["2026-01-01-old-change"]);
  });

  it("includes spec details, raw markdown, artifacts, and related warnings", async () => {
    const { model } = await buildFixtureModel("warnings-openspec");

    expect(model.specs[0].markdown).toContain("### Requirement: Missing scenario");
    expect(model.specs[0].warnings.map((warning) => warning.code)).toEqual([
      "missing-purpose",
      "requirement-missing-scenario",
      "malformed-scenario-heading",
    ]);
    expect(model.activeChanges[0].artifacts.map((artifact) => artifact.name)).toEqual(["Proposal"]);
    expect(model.activeChanges[0].warnings.map((warning) => warning.code)).toEqual([
      "missing-change-artifact",
      "missing-change-artifact",
    ]);
  });
});

describe("viewer rendering", () => {
  it("renders dashboard, spec, change, archive, and warning pages", async () => {
    const { model } = await buildFixtureModel("changes-openspec");
    const dashboard = renderRoute(model, "/").html;

    expect(dashboard).toContain("OpenSpec Dashboard");
    expect(dashboard).toContain("Project health: no analyzer warnings found.");
    expect(dashboard).toContain("Current specs");
    expect(dashboard).toContain("Active changes");
    expect(dashboard).toContain("Archived changes");
    expect(dashboard).toContain("Requirements");
    expect(dashboard).toContain("Scenarios");
    expect(dashboard).toContain("Warnings");
    expect(dashboard).toContain("Lifecycle guidance");
    expect(dashboard).toContain("Explore → Propose → Review → Apply → Archive");
    expect(dashboard).toContain("Active change lifecycle queue");
    expect(dashboard).toContain("Review triage");
    expect(dashboard).toContain("Review");
    expect(dashboard).toContain("Next action:");
    expect(dashboard).toContain("/changes/add-core/review");
    expect(dashboard).toContain("Tasks: 1/2 complete");
    expect(dashboard).toContain("Warnings: 0");
    expect(dashboard).toContain("Artifact completeness");
    expect(dashboard).toContain("Recent spec preview");
    expect(dashboard).toContain("Showing 1 of 1 current spec(s).");
    expect(dashboard).toContain("1 requirement(s), 1 scenario(s), 0 warning(s)");
    expect(dashboard).toContain("Recent archive preview");
    expect(dashboard).toContain("Showing 1 of 1 archived change(s).");
    expect(dashboard).toContain("2026-01-01-old-change");
    expect(dashboard).toContain("Warning summary");
    expect(dashboard).toContain("No warnings found. Project health checks are clear.");
    expect(dashboard).toContain("Project verification");
    expect(dashboard).toContain("Health & Paths");
    expect(renderRoute(model, "/specs").html).toContain("openspec/specs/core/spec.md");
    expect(renderRoute(model, "/specs/core").html).toContain("Core behavior MUST exist.");
    expect(renderRoute(model, "/changes").html).toContain("add-core");
    expect(renderRoute(model, "/changes").html).toContain("Next action:");
    expect(renderRoute(model, "/changes/add-core").html).toContain("Spec Delta: core");
    expect(renderRoute(model, "/changes/add-core").html).toContain("Recommended next actions");
    expect(renderRoute(model, "/archive").html).toContain("2026-01-01-old-change");
    expect(renderRoute(model, "/archive/2026-01-01-old-change").html).toContain("Old change.");
    expect(renderRoute(model, "/archive/2026-01-01-old-change").html).toContain(
      "Lifecycle complete",
    );
    expect(renderRoute(model, "/warnings").html).toContain("Warnings");
  });

  it("renders dashboard warnings and empty states", async () => {
    const warningsModel = (await buildFixtureModel("warnings-openspec")).model;
    const warningsDashboard = renderRoute(warningsModel, "/").html;

    expect(warningsDashboard).toContain("Project health: 5 warning(s) need attention");
    expect(warningsDashboard).toContain("5 warning(s) found.");
    expect(warningsDashboard).toContain("missing-purpose");
    expect(warningsDashboard).toContain("incomplete");
    expect(warningsDashboard).toContain("Warnings: 2");
    expect(warningsDashboard).toContain("— design");
    expect(warningsDashboard).toContain("— tasks");
    expect(warningsDashboard).toContain("No archived changes found.");

    const minimalDashboard = renderRoute(
      (await buildFixtureModel("minimal-openspec")).model,
      "/",
    ).html;
    expect(minimalDashboard).toContain("No active change review queue.");
    expect(minimalDashboard).toContain("No active OpenSpec changes were found.");
    expect(minimalDashboard).toContain("Explore next");
    expect(minimalDashboard).toContain("openspec list");
    expect(minimalDashboard).toContain("No archived changes found.");
    expect(minimalDashboard).toContain("No warnings found. Project health checks are clear.");
  });

  it("prioritizes active changes and limits dashboard spec and archive previews", async () => {
    const { model } = await buildFixtureModel("changes-openspec");
    const specTemplate = model.specs[0];
    const archiveTemplate = model.archivedChanges[0];
    const expandedModel = {
      ...model,
      summary: { ...model.summary, currentSpecs: 5, archivedChanges: 5 },
      specs: ["alpha", "bravo", "charlie", "delta", "echo"].map((name) => ({
        ...specTemplate,
        name,
        relativePath: `openspec/specs/${name}/spec.md`,
        path: `${model.rootPath}/openspec/specs/${name}/spec.md`,
      })),
      archivedChanges: [
        "2026-01-01-old-change",
        "2026-02-01-middle-change",
        "2026-03-01-new-change",
        "2025-12-01-older-change",
        "2024-01-01-oldest-change",
      ].map((name) => ({
        ...archiveTemplate,
        name,
        relativePath: `openspec/changes/archive/${name}`,
        path: `${model.rootPath}/openspec/changes/archive/${name}`,
      })),
    };

    const dashboard = renderRoute(expandedModel, "/").html;
    expect(dashboard.indexOf("Review triage")).toBeLessThan(
      dashboard.indexOf("Recent spec preview"),
    );
    expect(dashboard.indexOf("Review triage")).toBeLessThan(
      dashboard.indexOf("Recent archive preview"),
    );
    expect(dashboard).toContain("View all specs (5)");
    expect(dashboard).toContain("Showing 3 of 5 current spec(s).");
    expect(dashboard).toContain("openspec/specs/echo/spec.md");
    expect(dashboard).toContain("openspec/specs/delta/spec.md");
    expect(dashboard).toContain("openspec/specs/charlie/spec.md");
    expect(dashboard).not.toContain("openspec/specs/bravo/spec.md");
    expect(dashboard).not.toContain("openspec/specs/alpha/spec.md");
    expect(dashboard).toContain("View all archived changes (5)");
    expect(dashboard).toContain("Showing 3 of 5 archived change(s).");
    expect(dashboard).toContain("2026-03-01-new-change");
    expect(dashboard).toContain("2026-02-01-middle-change");
    expect(dashboard).toContain("2026-01-01-old-change");
    expect(dashboard).not.toContain("2025-12-01-older-change");
    expect(dashboard).not.toContain("2024-01-01-oldest-change");

    const specsIndex = renderRoute(expandedModel, "/specs").html;
    expect(specsIndex).toContain("openspec/specs/alpha/spec.md");
    expect(specsIndex).toContain("openspec/specs/bravo/spec.md");
    const archiveIndex = renderRoute(expandedModel, "/archive").html;
    expect(archiveIndex).toContain("2025-12-01-older-change");
    expect(archiveIndex).toContain("2024-01-01-oldest-change");
  });

  it("renders persistent navigation on dashboard and section pages", async () => {
    const { model } = await buildFixtureModel("changes-openspec");
    const paths = [
      "/",
      "/specs",
      "/specs/core",
      "/changes",
      "/changes/add-core",
      "/changes/add-core/review",
      "/archive",
      "/archive/2026-01-01-old-change",
      "/warnings",
    ];

    for (const path of paths) {
      const page = renderRoute(model, path).html;
      expect(page).toContain('aria-label="Viewer sections"');
      expect(page).toContain('<a href="/">Dashboard</a>');
      expect(page).toContain('href="/specs">Specs <span class="nav-count">1</span>');
      expect(page).toContain('href="/changes">Active Changes <span class="nav-count">1</span>');
      expect(page).toContain('href="/archive">Archive <span class="nav-count">1</span>');
      expect(page).toContain('href="/warnings">Warnings <span class="nav-count">0</span>');
    }
  });

  it("renders a read-only active change review route when companion mode is disabled", async () => {
    const { model } = await buildFixtureModel("changes-openspec");
    const detail = renderRoute(model, "/changes/add-core");
    const review = renderRoute(model, "/changes/add-core/review");

    expect(detail.html).toContain("/changes/add-core/review");
    expect(detail.html).toContain("Review this active change");
    expect(detail.html).not.toContain("/changes/add-core/companion");
    expect(review.status).toBe(200);
    expect(review.html).toContain("Pi-assisted review");
    expect(review.html).toContain("Add core behavior.");
    expect(review.html).toContain("Initial design.");
    expect(review.html).toContain("Second task");
    expect(review.html).toContain("Spec Delta: core");
    expect(review.html).toContain("Artifact completion details");
    expect(review.html).toContain("Lifecycle guidance");
    expect(review.html).toContain("Recommended next actions");
    expect(review.html).toContain("openspec validate add-core");
    expect(review.html).toContain("Local lifecycle buttons can run allowlisted commands");
    expect(review.html).toContain("data-lifecycle-action");
    expect(review.html).toContain("Task state actions");
    expect(review.html).toContain("Mark complete");
    expect(review.html).toContain("/api/lifecycle-actions");
    expect(review.html).toContain("Review companion mode is disabled for this viewer session");
    expect(review.html).toContain("Browser feedback writes");
    expect(review.html).not.toContain("/changes/add-core/companion");
    expect(review.html).not.toContain("Select text in this artifact");
    expect(review.html).not.toContain("<textarea");
    expect(review.html).not.toContain('type="submit"');
    expect(review.html).not.toContain("Save comment");
  });

  it("renders companion navigation and document-first review companion page when enabled", async () => {
    const { parsed, analysis } = await buildFixtureModel("changes-openspec");
    const model = await buildViewerModelFromProject(parsed, analysis);
    const detail = renderRoute(model, "/changes/add-core");
    const review = renderRoute(model, "/changes/add-core/review");
    const companion = renderRoute(model, "/changes/add-core/companion");

    expect(detail.html).toContain("/changes/add-core/companion");
    expect(review.html).toContain("Open review companion");
    expect(companion.status).toBe(200);
    expect(companion.html).toContain("Document-first review companion");
    expect(companion.html).toContain('main class="companion-page"');
    expect(companion.html).toContain("companion-shell");
    expect(companion.html).toContain("companion-document");
    expect(companion.html).not.toContain("companion-panel");
    expect(companion.html).not.toContain("Capture feedback");
    expect(companion.html).not.toContain('name="selectedText" rows=');
    expect(companion.html).toContain('role="tablist"');
    expect(companion.html).toContain('data-stage-tab="proposal"');
    expect(companion.html).toContain('data-stage-tab="design"');
    expect(companion.html).toContain('data-stage-tab="tasks"');
    expect(companion.html).toContain('data-stage-tab="spec-deltas"');
    expect(companion.html).toContain("Spec Delta: core");
    expect(companion.html).toContain("Review guidance");
    expect(companion.html).toContain("Lifecycle-aware companion guidance");
    expect(companion.html).toContain("Comments and Pi analysis remain guidance");
    expect(companion.html).toContain("Local lifecycle buttons can run allowlisted commands");
    expect(companion.html).toContain("data-lifecycle-action");
    expect(companion.html).toContain("/api/lifecycle-actions");
    expect(companion.html).toContain("comment-composer");
    expect(companion.html).toContain("annotation-affordance");
    expect(companion.html).toContain("comments-overview");
    expect(companion.html).toContain("thread-popover");
    expect(companion.html).toContain("Save feedback record");
    expect(companion.html).toContain("do not directly edit proposal, design");
    expect(companion.html).toContain("Add core behavior.");
    expect(companion.html).toContain("data-comment-marker");
    expect(companion.html).toContain("data-open-thread");
    expect(companion.html).toContain("data-delete-comment");
    expect(companion.html).toContain('method: "DELETE"');
    expect(companion.html).toContain("Delete this comment and all replies");
    expect(companion.html).toContain("contextmenu");
    expect(companion.html).toContain("Ask Pi");
    expect(companion.html).toContain("Live review analysis");
    expect(companion.html).toContain("Context that will be sent to Pi");
    expect(companion.html).toContain("data-review-agent-start");
    expect(companion.html).toContain("review-agent-transcript");
    expect(companion.html).toContain("data-review-agent-follow-up");
  });

  it("renders disabled and unavailable review agent states without removing comments UI", async () => {
    const { parsed, analysis } = await buildFixtureModel("changes-openspec");
    const disabled = await buildViewerModelFromProject(parsed, analysis, {
      reviewAgentEnabled: false,
    });
    const unavailable = await buildViewerModelFromProject(parsed, analysis, {
      reviewAgentUnavailableReason: "No model credentials are configured.",
    });

    const disabledCompanion = renderRoute(disabled, "/changes/add-core/companion").html;
    expect(disabledCompanion).toContain("Pi review analysis disabled");
    expect(disabledCompanion).toContain(
      "Sidecar comment creation and thread review remain available",
    );
    expect(disabledCompanion).toContain("comment-composer");
    expect(disabledCompanion).not.toContain('<button type="button" data-review-agent-start');

    const unavailableCompanion = renderRoute(unavailable, "/changes/add-core/companion").html;
    expect(unavailableCompanion).toContain("Pi review analysis unavailable");
    expect(unavailableCompanion).toContain("No model credentials are configured.");
    expect(unavailableCompanion).toContain("comments-overview");
    expect(unavailableCompanion).not.toContain('<button type="button" data-review-agent-start');
  });

  it("renders marker and overview JavaScript states for exact, fuzzy, orphaned, and missing comments", async () => {
    const { parsed, analysis } = await buildFixtureModel("changes-openspec");
    const model = await buildViewerModelFromProject(parsed, analysis);
    const companion = renderRoute(model, "/changes/add-core/companion").html;

    expect(companion).toContain('record.anchorState !== "exact" && record.anchorState !== "fuzzy"');
    expect(companion).toContain('record.anchorState === "exact" || record.anchorState === "fuzzy"');
    expect(companion).toContain('escapeText(record.anchorState || "orphaned")');
    expect(companion).toContain("findArtifact(record.artifactPath)");
    expect(companion).toContain("Jump to marker");
    expect(companion).toContain("Inspect thread");
    expect(companion).toContain("Comments overview");
  });

  it("does not focus or scroll to the composer during selection and right-click handling", async () => {
    const { parsed, analysis } = await buildFixtureModel("changes-openspec");
    const model = await buildViewerModelFromProject(parsed, analysis);
    const companion = renderRoute(model, "/changes/add-core/companion").html;
    const selectionHandler = companion.slice(
      companion.indexOf('document.addEventListener("selectionchange"'),
      companion.indexOf('form.addEventListener("submit"'),
    );

    expect(selectionHandler).not.toContain("scrollIntoView");
    expect(selectionHandler).not.toContain(".focus(");
    expect(selectionHandler).toContain("captureSelection(context)");
    expect(selectionHandler).toContain("event.preventDefault()");
  });

  it("renders empty tab states for missing companion stages", async () => {
    const { parsed, analysis } = await buildFixtureModel("warnings-openspec");
    const model = await buildViewerModelFromProject(parsed, analysis);
    const companion = renderRoute(model, "/changes/incomplete/companion");

    expect(companion.status).toBe(200);
    expect(companion.html).toContain("No design artifact is available");
    expect(companion.html).toContain("No tasks artifact is available");
    expect(companion.html).toContain("No spec deltas artifact is available");
  });

  it("does not expose the companion route when companion mode is disabled", async () => {
    const { model } = await buildFixtureModel("changes-openspec");
    const companion = renderRoute(model, "/changes/add-core/companion");

    expect(companion.status).toBe(404);
    expect(companion.html).toContain("Review companion mode is not enabled");
  });

  it("escapes rendered markdown and returns not found pages", async () => {
    const { model } = await buildFixtureModel("minimal-openspec");

    expect(renderRoute(model, "/specs/widgets").html).toContain("Widget summary");
    expect(renderRoute(model, "/").html).not.toContain("/api/model-version");
    const notFound = renderRoute(model, "/missing");
    expect(notFound.status).toBe(404);
    expect(notFound.html).toContain("Not Found");
  });
});

describe("viewer server", () => {
  it("starts and serves browser HTML", async () => {
    const server = await startViewerServer({ root: fixturePath("minimal-openspec"), port: 0 });
    try {
      const response = await fetch(server.url);
      const body = await response.text();

      expect(response.status).toBe(200);
      expect(body).toContain("OpenSpec Dashboard");
      expect(body).toContain("Current specs");
    } finally {
      await server.close();
    }
  });

  it("exposes model version and watch polling only when watch mode is enabled", async () => {
    const watchedServer = await startViewerServer({
      root: fixturePath("minimal-openspec"),
      port: 0,
      watch: true,
    });
    try {
      const page = await fetch(watchedServer.url);
      const body = await page.text();
      const version = await fetch(`${watchedServer.url}/api/model-version`);
      const payload = (await version.json()) as { version: number };

      expect(body).toContain("/api/model-version");
      expect(version.status).toBe(200);
      expect(payload.version).toBe(1);
    } finally {
      await watchedServer.close();
    }

    const nonWatchedServer = await startViewerServer({
      root: fixturePath("minimal-openspec"),
      port: 0,
    });
    try {
      const page = await fetch(nonWatchedServer.url);
      const body = await page.text();
      const version = await fetch(`${nonWatchedServer.url}/api/model-version`);

      expect(body).not.toContain("/api/model-version");
      expect(version.status).toBe(404);
    } finally {
      await nonWatchedServer.close();
    }
  });

  it("refreshes the served model after watched spec and change artifact updates", async () => {
    const root = await copyFixtureToTemp("changes-openspec");
    const server = await startViewerServer({ root, port: 0, watch: true });
    try {
      await writeFile(
        path.join(root, "openspec", "specs", "core", "spec.md"),
        "## Purpose\n\nUpdated core purpose.\n\n### Requirement: Core behavior\n#### Scenario: Updated scenario\n- **WHEN** it runs\n- **THEN** it reflects changes\n",
        "utf8",
      );
      await waitForModelVersion(server.url, 2);
      const spec = await fetch(`${server.url}/specs/core`);
      expect(await spec.text()).toContain("Updated core purpose.");

      await writeFile(
        path.join(root, "openspec", "changes", "add-core", "proposal.md"),
        "## Why\n\nUpdated proposal text.\n",
        "utf8",
      );
      await waitForModelVersion(server.url, 3);
      const change = await fetch(`${server.url}/changes/add-core`);
      expect(await change.text()).toContain("Updated proposal text.");
    } finally {
      await server.close();
      await rm(root, { recursive: true, force: true });
    }
  });

  it("keeps serving the last successful model when watch refresh fails", async () => {
    const root = await copyFixtureToTemp("changes-openspec");
    const server = await startViewerServer({ root, port: 0, watch: true });
    try {
      await rm(path.join(root, "openspec", "config.yaml"));
      await writeFile(path.join(root, "openspec", "specs", "core", "spec.md"), "broken", "utf8");
      const payload = await waitForRefreshError(server.url);
      expect(payload.version).toBe(1);
      expect(payload.refreshError).toContain("Missing OpenSpec config");

      const response = await fetch(`${server.url}/specs/core`);
      const body = await response.text();
      expect(response.status).toBe(200);
      expect(body).toContain("Core behavior MUST exist.");
      expect(body).toContain("Latest watch refresh failed");
      expect(body).toContain("Serving the last successfully built OpenSpec model");
    } finally {
      await server.close();
      await rm(root, { recursive: true, force: true });
    }
  });

  it("does not modify OpenSpec artifacts when watch mode refreshes", async () => {
    const root = await copyFixtureToTemp("changes-openspec");
    const proposalPath = path.join(root, "openspec", "changes", "add-core", "proposal.md");
    const proposalBefore = await readFile(proposalPath, "utf8");
    const server = await startViewerServer({ root, port: 0, watch: true });
    try {
      await writeFile(
        path.join(root, "openspec", "specs", "core", "spec.md"),
        "## Purpose\n\nUpdated only by the test.\n\n### Requirement: Core behavior\n#### Scenario: Updated scenario\n- **WHEN** it runs\n- **THEN** it reflects changes\n",
        "utf8",
      );
      await waitForModelVersion(server.url, 2);
      expect(await readFile(proposalPath, "utf8")).toBe(proposalBefore);
    } finally {
      await server.close();
      await rm(root, { recursive: true, force: true });
    }
  });

  it("watches review companion sidecar feedback when companion mode is enabled", async () => {
    const root = await copyFixtureToTemp("changes-openspec");
    const server = await startViewerServer({ root, port: 0, watch: true, reviewCompanion: true });
    try {
      const response = await fetch(`${server.url}/api/review-feedback/add-core`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          body: "Refresh after sidecar feedback.",
          artifactPath: "openspec/changes/add-core/proposal.md",
          selectedText: "Add core behavior.",
        }),
      });
      expect(response.status).toBe(201);
      await waitForModelVersion(server.url, 2);
    } finally {
      await server.close();
      await rm(root, { recursive: true, force: true });
    }
  });

  it("creates browser review agent sessions through a stub Pi bridge without credentials", async () => {
    const root = await copyFixtureToTemp("changes-openspec");
    const openspecBefore = await snapshotTree(path.join(root, "openspec"));
    const bridge: PiReviewBridge = {
      async start(record) {
        const listeners = new Set<(event: BrowserReviewAgentEvent) => void>();
        const handle: PiReviewSessionHandle = {
          id: record.id,
          events: [],
          subscribe(listener) {
            listeners.add(listener);
            for (const event of handle.events) listener(event);
            return () => listeners.delete(listener);
          },
          async followUp() {},
          async abort() {},
        };
        queueMicrotask(() => {
          const event = {
            type: "session_finished" as const,
            sessionId: record.id,
            responseText: "Stubbed Pi response.",
          };
          handle.events.push(event);
          for (const listener of listeners) listener(event);
        });
        return handle;
      },
    };
    const server = await startViewerServer({ root, port: 0, reviewAgentBridge: bridge });
    try {
      const response = await fetch(`${server.url}/api/review-agent/add-core/sessions`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ reviewerInstruction: "Use stub bridge." }),
      });
      const payload = (await response.json()) as { session: { id: string }; eventsUrl: string };
      expect(response.status).toBe(201);
      expect(payload.eventsUrl).toContain(
        `/api/review-agent/sessions/${payload.session.id}/events`,
      );

      const recovered = await waitForReviewAgentSession(server.url, "add-core", payload.session.id);
      expect(recovered.responseText).toBe("Stubbed Pi response.");
      expect(await snapshotTree(path.join(root, "openspec"))).toEqual(openspecBefore);
    } finally {
      await server.close();
      await rm(root, { recursive: true, force: true });
    }
  });

  it("rejects feedback writes when review companion mode is disabled", async () => {
    const root = await copyFixtureToTemp("changes-openspec");
    const server = await startViewerServer({ root, port: 0, reviewCompanion: false });
    try {
      const response = await fetch(`${server.url}/api/review-feedback/add-core`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ body: "Needs more detail." }),
      });

      expect(response.status).toBe(403);
      expect(
        existsSync(path.join(root, ".pi", "openspec-review", "comments", "add-core.jsonl")),
      ).toBe(false);
    } finally {
      await server.close();
      await rm(root, { recursive: true, force: true });
    }
  });

  it("hard deletes companion feedback threads from sidecar storage", async () => {
    const root = await copyFixtureToTemp("changes-openspec");
    const openspecBefore = await snapshotTree(path.join(root, "openspec"));
    const server = await startViewerServer({ root, port: 0 });
    try {
      const response = await fetch(`${server.url}/api/review-feedback/add-core`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          body: "Delete this thread.",
          artifactPath: "openspec/changes/add-core/proposal.md",
          selectedText: "Add core behavior.",
        }),
      });
      const payload = (await response.json()) as { feedback: { id: string } };
      expect(response.status).toBe(201);

      const replyResponse = await fetch(
        `${server.url}/api/review-feedback/add-core/${payload.feedback.id}/reply`,
        {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({ body: "Reply to delete too." }),
        },
      );
      expect(replyResponse.status).toBe(201);

      const deleteResponse = await fetch(
        `${server.url}/api/review-feedback/add-core/${payload.feedback.id}`,
        { method: "DELETE" },
      );
      expect(deleteResponse.status).toBe(200);

      const list = await fetch(`${server.url}/api/review-feedback/add-core`);
      const listPayload = (await list.json()) as { feedback: unknown[] };
      expect(listPayload.feedback).toHaveLength(0);

      const sidecarPath = path.join(root, ".pi", "openspec-review", "comments", "add-core.jsonl");
      const sidecar = await readFile(sidecarPath, "utf8");
      expect(sidecar).not.toContain("Delete this thread.");
      expect(sidecar).not.toContain("Reply to delete too.");
      expect(await snapshotTree(path.join(root, "openspec"))).toEqual(openspecBefore);
    } finally {
      await server.close();
      await rm(root, { recursive: true, force: true });
    }
  });

  it("writes and edits companion feedback only in sidecar storage for active changes by default", async () => {
    const root = await copyFixtureToTemp("changes-openspec");
    const openspecBefore = await snapshotTree(path.join(root, "openspec"));
    const server = await startViewerServer({ root, port: 0 });
    try {
      const response = await fetch(`${server.url}/api/review-feedback/add-core`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          body: "Clarify the proposal impact.",
          artifactPath: "openspec/changes/add-core/proposal.md",
          selectedText: "Add core behavior.",
          category: "clarity",
          severity: "suggestion",
        }),
      });
      const payload = (await response.json()) as {
        feedback: { id: string; body: string; changeName: string };
      };

      expect(response.status).toBe(201);
      expect(payload.feedback.changeName).toBe("add-core");
      expect(payload.feedback.body).toBe("Clarify the proposal impact.");

      const editResponse = await fetch(
        `${server.url}/api/review-feedback/add-core/${payload.feedback.id}/edit`,
        {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({ body: "Edited proposal impact comment." }),
        },
      );
      expect(editResponse.status).toBe(201);

      const emptyEdit = await fetch(
        `${server.url}/api/review-feedback/add-core/${payload.feedback.id}/edit`,
        {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({ body: "" }),
        },
      );
      expect(emptyEdit.status).toBe(400);

      const list = await fetch(`${server.url}/api/review-feedback/add-core`);
      const listPayload = (await list.json()) as {
        feedback: Array<{ body: string; editHistory: unknown[] }>;
      };
      expect(listPayload.feedback[0].body).toBe("Edited proposal impact comment.");
      expect(listPayload.feedback[0].editHistory).toHaveLength(1);

      const sidecarPath = path.join(root, ".pi", "openspec-review", "comments", "add-core.jsonl");
      const sidecar = await readFile(sidecarPath, "utf8");
      expect(sidecar).toContain("Clarify the proposal impact.");
      expect(sidecar).toContain("Edited proposal impact comment.");
      expect(await snapshotTree(path.join(root, "openspec"))).toEqual(openspecBefore);
    } finally {
      await server.close();
      await rm(root, { recursive: true, force: true });
    }
  });

  it("validates review agent session requests for active safe changes and rejects invalid targets", async () => {
    const { parsed, analysis } = await buildFixtureModel("changes-openspec");
    const enabled = await buildViewerModelFromProject(parsed, analysis);
    const disabled = await buildViewerModelFromProject(parsed, analysis, {
      reviewAgentEnabled: false,
    });

    await expect(
      validateReviewAgentSessionRequest(enabled, "add-core", {
        reviewerInstruction: "Review selected feedback.",
      }),
    ).resolves.toMatchObject({ ok: true, reviewerInstruction: "Review selected feedback." });
    await expect(
      validateReviewAgentSessionRequest(disabled, "add-core", {}),
    ).resolves.toMatchObject({ ok: false, status: 403 });
    await expect(
      validateReviewAgentSessionRequest(enabled, "missing-change", {}),
    ).resolves.toMatchObject({
      ok: false,
      status: 404,
    });
    await expect(
      validateReviewAgentSessionRequest(enabled, "2026-01-01-old-change", {}),
    ).resolves.toMatchObject({ ok: false, status: 404 });
    await expect(
      validateReviewAgentSessionRequest(enabled, "../add-core", {}),
    ).resolves.toMatchObject({
      ok: false,
      status: 400,
    });
    await expect(
      validateReviewAgentSessionRequest(enabled, "add-core", { selectedCommentIds: ["missing"] }),
    ).resolves.toMatchObject({ ok: false, status: 400 });
  });

  it("creates scoped review agent bundles and recovers completed sessions without OpenSpec mutation", async () => {
    const root = await copyFixtureToTemp("changes-openspec");
    const openspecBefore = await snapshotTree(path.join(root, "openspec"));
    const scan = await scanProject(root);
    if (!scan.ok) throw new Error(scan.fatalError.message);
    const parsed = await parseProject(scan.project);
    const model = await buildViewerModelFromProject(parsed, analyzeProject(parsed));
    const change = model.activeChanges[0];
    const createdAt = "2026-01-01T00:00:00.000Z";
    const feedback = [
      {
        id: "selected",
        changeName: change.name,
        artifactPath: change.artifacts[0].relativePath,
        anchor: {
          artifactPath: change.artifacts[0].relativePath,
          selectedText: "Add core behavior.",
        },
        body: "Selected feedback.",
        source: "browser" as const,
        status: "open" as const,
        createdAt,
        updatedAt: createdAt,
        replies: [],
        statusHistory: [],
        editHistory: [],
        anchorState: "exact" as const,
      },
      {
        id: "unselected",
        changeName: change.name,
        artifactPath: change.artifacts[0].relativePath,
        anchor: {
          artifactPath: change.artifacts[0].relativePath,
          selectedText: "Add core behavior.",
        },
        body: "Unselected feedback.",
        source: "browser" as const,
        status: "open" as const,
        createdAt,
        updatedAt: createdAt,
        replies: [],
        statusHistory: [],
        editHistory: [],
        anchorState: "exact" as const,
      },
    ];
    const bundle = createReviewAgentBundle({
      id: "bundle-1",
      changeName: change.name,
      reviewerInstruction: "Focus on selected feedback.",
      selectedCommentIds: ["selected"],
      includeAllOpenFeedback: false,
      artifacts: change.artifacts,
      warnings: change.warnings,
      feedback,
      createdAt,
    });
    expect(bundle.feedback.map((thread) => thread.id)).toEqual(["selected"]);
    expect(bundle.artifacts[0].sha256).toHaveLength(64);

    const session = await createBrowserReviewAgentSession(model, "add-core", {
      reviewerInstruction: "Recover this session.",
    });
    await appendReviewAgentResponseCompleted(root, "add-core", session.id, "Completed response.");
    const recovered = await listRecentReviewAgentSessions(root, "add-core");
    expect(recovered[0]).toMatchObject({
      id: session.id,
      status: "completed",
      responseText: "Completed response.",
    });
    expect(await snapshotTree(path.join(root, "openspec"))).toEqual(openspecBefore);
    await rm(root, { recursive: true, force: true });
  });

  it("rejects missing, archived, and path-traversal feedback write requests", async () => {
    const root = await copyFixtureToTemp("changes-openspec");
    const server = await startViewerServer({ root, port: 0, reviewCompanion: true });
    try {
      const invalidArtifact = await fetch(`${server.url}/api/review-feedback/add-core`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          body: "Nope.",
          artifactPath: "openspec/changes/add-core/missing.md",
          selectedText: "Nope.",
        }),
      });
      const invalidSelection = await fetch(`${server.url}/api/review-feedback/add-core`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          body: "Nope.",
          artifactPath: "openspec/changes/add-core/proposal.md",
          selectedText: "Text that is not present.",
        }),
      });
      const missing = await fetch(`${server.url}/api/review-feedback/missing-change`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ body: "Nope." }),
      });
      const archived = await fetch(`${server.url}/api/review-feedback/2026-01-01-old-change`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ body: "Nope." }),
      });
      const traversal = await fetch(`${server.url}/api/review-feedback/..%2Fadd-core`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ body: "Nope." }),
      });

      expect(invalidArtifact.status).toBe(400);
      expect(invalidSelection.status).toBe(400);
      expect(missing.status).toBe(404);
      expect(archived.status).toBe(404);
      expect(traversal.status).toBe(400);
      expect(
        existsSync(path.join(root, ".pi", "openspec-review", "comments", "missing-change.jsonl")),
      ).toBe(false);
    } finally {
      await server.close();
      await rm(root, { recursive: true, force: true });
    }
  });

  it("fails before starting when the OpenSpec config is missing", async () => {
    await expect(
      startViewerServer({ root: fixturePath("does-not-exist"), port: 0 }),
    ).rejects.toThrow("Missing OpenSpec config");
  });
});

async function copyFixtureToTemp(name: string): Promise<string> {
  const root = await mkdtemp(path.join(os.tmpdir(), "openspec-doc-"));
  await copyFixture(name, root);
  return root;
}

async function snapshotTree(directory: string): Promise<Record<string, string>> {
  const entries = await readdir(directory, { withFileTypes: true });
  const snapshot: Record<string, string> = {};

  for (const entry of entries) {
    const entryPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      const childSnapshot = await snapshotTree(entryPath);
      for (const [childPath, content] of Object.entries(childSnapshot)) {
        snapshot[`${entry.name}/${childPath}`] = content;
      }
    } else if (entry.isFile()) {
      snapshot[entry.name] = await readFile(entryPath, "utf8");
    }
  }

  return snapshot;
}

async function waitForModelVersion(
  url: string,
  expectedVersion: number,
): Promise<{ version: number }> {
  return waitForVersionPayload(url, (payload) => payload.version >= expectedVersion);
}

async function waitForRefreshError(
  url: string,
): Promise<{ version: number; refreshError?: string }> {
  return waitForVersionPayload(url, (payload) => Boolean(payload.refreshError));
}

async function waitForReviewAgentSession(
  url: string,
  changeName: string,
  sessionId: string,
): Promise<{ id: string; status: string; responseText?: string }> {
  const deadline = Date.now() + 5000;
  let lastPayload: { sessions?: Array<{ id: string; status: string; responseText?: string }> } = {};

  while (Date.now() < deadline) {
    const response = await fetch(`${url}/api/review-agent/${changeName}/sessions`);
    lastPayload = (await response.json()) as typeof lastPayload;
    const session = lastPayload.sessions?.find((item) => item.id === sessionId);
    if (session?.status === "completed") {
      return session;
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
  }

  throw new Error(
    `Timed out waiting for review agent session. Last payload: ${JSON.stringify(lastPayload)}`,
  );
}

async function waitForVersionPayload(
  url: string,
  predicate: (payload: { version: number; refreshError?: string }) => boolean,
): Promise<{ version: number; refreshError?: string }> {
  const deadline = Date.now() + 5000;
  let lastPayload: { version: number; refreshError?: string } | undefined;

  while (Date.now() < deadline) {
    const response = await fetch(`${url}/api/model-version`);
    lastPayload = (await response.json()) as { version: number; refreshError?: string };
    if (predicate(lastPayload)) {
      return lastPayload;
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
  }

  throw new Error(
    `Timed out waiting for model version payload. Last payload: ${JSON.stringify(lastPayload)}`,
  );
}
