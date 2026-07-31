import type { LifecycleAction, LifecycleGuidance, LifecyclePhase } from "../core/lifecycle.js";
import type { ProjectWarning } from "../core/types.js";
import type { ViewerArtifact, ViewerChange, ViewerModel, ViewerSpec } from "./model.js";

const DASHBOARD_PREVIEW_LIMIT = 3;
const COMPANION_STAGES = [
  { id: "proposal", label: "Proposal" },
  { id: "design", label: "Design" },
  { id: "tasks", label: "Tasks" },
  { id: "spec-deltas", label: "Spec Deltas" },
] as const;

type CompanionStageId = (typeof COMPANION_STAGES)[number]["id"];

export interface RenderedPage {
  status: number;
  html: string;
}

export function renderRoute(model: ViewerModel, routePath: string): RenderedPage {
  const path = normalizeRoute(routePath);

  if (path === "/") {
    return ok(renderDashboard(model));
  }
  if (path === "/specs") {
    return ok(renderSpecsIndex(model));
  }
  if (path.startsWith("/specs/")) {
    const name = decodeURIComponent(path.slice("/specs/".length));
    const spec = model.specs.find((item) => item.name === name);
    return spec ? ok(renderSpecDetail(model, spec)) : notFound(model, `Spec not found: ${name}`);
  }
  if (path === "/changes") {
    return ok(renderChangesIndex(model));
  }
  if (path.startsWith("/changes/")) {
    const changePath = path.slice("/changes/".length);
    if (changePath.endsWith("/review")) {
      const name = decodeURIComponent(changePath.slice(0, -"/review".length));
      const change = model.activeChanges.find((item) => item.name === name);
      return change
        ? ok(renderChangeReview(model, change))
        : notFound(model, `Active change not found: ${name}`);
    }
    if (changePath.endsWith("/companion")) {
      const name = decodeURIComponent(changePath.slice(0, -"/companion".length));
      const change = model.activeChanges.find((item) => item.name === name);
      if (!change) {
        return notFound(model, `Active change not found: ${name}`);
      }
      return model.reviewCompanionEnabled
        ? ok(renderChangeCompanion(model, change))
        : notFound(model, `Review companion mode is not enabled for: ${name}`);
    }

    const name = decodeURIComponent(changePath);
    const change = model.activeChanges.find((item) => item.name === name);
    return change
      ? ok(renderChangeDetail(model, change))
      : notFound(model, `Active change not found: ${name}`);
  }
  if (path === "/archive") {
    return ok(renderArchiveIndex(model));
  }
  if (path.startsWith("/archive/")) {
    const name = decodeURIComponent(path.slice("/archive/".length));
    const change = model.archivedChanges.find((item) => item.name === name);
    return change
      ? ok(renderChangeDetail(model, change))
      : notFound(model, `Archived change not found: ${name}`);
  }
  if (path === "/warnings") {
    return ok(renderWarnings(model));
  }

  return notFound(model, `Page not found: ${path}`);
}

function renderDashboard(model: ViewerModel): string {
  const summary = model.summary;
  return layout(
    "OpenSpec Dashboard",
    model,
    html`
      <section class="hero dashboard-hero">
        <p class="eyebrow">OpenSpec project dashboard</p>
        <h1>OpenSpec Dashboard</h1>
        <p>Review active OpenSpec work first, then browse recent documentation and archive activity.</p>
      </section>
      <section class="cards dashboard-counts" aria-label="OpenSpec counts">
        ${countCard("Current specs", summary.currentSpecs, "/specs")}
        ${countCard("Active changes", summary.activeChanges, "/changes")}
        ${countCard("Archived changes", summary.archivedChanges, "/archive")}
        ${countCard("Requirements", summary.requirements)}
        ${countCard("Scenarios", summary.scenarios)}
        ${countCard("Warnings", summary.warnings, "/warnings")}
      </section>
      ${renderDashboardLifecycleOverview(model)}
      ${renderDashboardReviewTriage(model)}
      <div class="dashboard-grid secondary-dashboard-grid">
        ${renderDashboardSpecOverview(model)}
        ${renderDashboardArchiveSummary(model)}
        ${renderDashboardWarningSummary(model)}
        ${renderDashboardSupportMetadata(model)}
      </div>
    `,
  );
}

function renderDashboardLifecycleOverview(model: ViewerModel): string {
  return html`
    <section class="dashboard-section lifecycle-overview" aria-label="OpenSpec lifecycle guidance">
      <div class="section-heading">
        <div>
          <p class="eyebrow">Lifecycle guidance</p>
          <h2>Explore → Propose → Review → Apply → Archive</h2>
        </div>
        <span class="phase-badge phase-${model.lifecycle.phase.id}">${escapeHtml(model.lifecycle.phase.label)}</span>
      </div>
      <p>${escapeHtml(model.lifecycle.summary)}</p>
      ${renderLifecyclePhases(model.lifecycle.phases)}
      ${
        model.lifecycle.activeQueue.length === 0
          ? html`
              <div class="notice lifecycle-panel compact-panel">
                <h3>Explore next</h3>
                <p>Clarify an idea in Pi or your agent workflow before creating an OpenSpec proposal.</p>
                ${renderLifecycleActions(model.lifecycle.actions, model.reviewCompanionEnabled)}
              </div>
            `
          : html`
              <h3>Active change lifecycle queue</h3>
              <ul class="dashboard-list lifecycle-queue">
                ${model.lifecycle.activeQueue
                  .map((item) => renderLifecycleQueueItem(model, item.name, item.guidance))
                  .join("\n")}
              </ul>
            `
      }
      ${renderLifecycleSafetyNotes(model.lifecycle.safetyNotes)}
    </section>
  `;
}

function renderLifecyclePhases(phases: readonly LifecyclePhase[]): string {
  return html`
    <ol class="lifecycle-phases" aria-label="OpenSpec lifecycle phases">
      ${phases
        .map(
          (phase) => html`
            <li>
              <span class="phase-badge phase-${phase.id}">${escapeHtml(phase.label)}</span>
              <span>${escapeHtml(phase.description)}</span>
            </li>
          `,
        )
        .join("\n")}
    </ol>
  `;
}

function renderLifecycleQueueItem(
  model: ViewerModel,
  changeName: string,
  guidance: LifecycleGuidance,
): string {
  const reviewPath = `/changes/${encodeURIComponent(changeName)}/review`;
  return html`
    <li>
      <h3>
        <a href="${reviewPath}">${escapeHtml(changeName)}</a>
        ${renderPhaseBadge(guidance)}
      </h3>
      <p>${escapeHtml(guidance.summary)}</p>
      <p><strong>Next action:</strong> ${escapeHtml(primaryActionSummary(guidance))}</p>
      <p>
        <a href="${reviewPath}">Open review →</a>
        ${
          model.reviewCompanionEnabled
            ? html` · <a href="/changes/${encodeURIComponent(changeName)}/companion">Open companion →</a>`
            : ""
        }
      </p>
    </li>
  `;
}

function projectHealthSummary(model: ViewerModel): string {
  if (model.summary.warnings === 0) {
    return "Project health: no analyzer warnings found.";
  }

  return `Project health: ${model.summary.warnings} warning(s) need attention across the OpenSpec project.`;
}

function renderDashboardReviewTriage(model: ViewerModel): string {
  return html`
    <section class="dashboard-section dashboard-primary-section">
      <div class="section-heading">
        <div>
          <p class="eyebrow">Review triage</p>
          <h2>Active Changes</h2>
        </div>
        <a href="/changes">View all active changes (${model.activeChanges.length}) →</a>
      </div>
      ${
        model.activeChanges.length === 0
          ? html`<p>No active change review queue.</p>`
          : html`<ul class="dashboard-list">
              ${[...model.activeChanges].sort(compareChangesForTriage).map(renderDashboardChange).join("\n")}
            </ul>`
      }
    </section>
  `;
}

function renderDashboardChange(change: ViewerChange): string {
  return html`
    <li>
      <h3><a href="/changes/${encodeURIComponent(change.name)}/review">${escapeHtml(change.name)}</a> ${warningBadge(change.warnings)}</h3>
      <p class="muted">${escapeHtml(change.relativePath)}</p>
      ${renderPhaseBadge(change.lifecycle)}
      <p><strong>Next action:</strong> ${escapeHtml(primaryActionSummary(change.lifecycle))}</p>
      <p>Tasks: ${formatTaskProgress(change)}</p>
      <p>Warnings: ${change.warnings.length}</p>
      <p>Artifact completeness: ${artifactFlags(change)}</p>
      <p><a href="/changes/${encodeURIComponent(change.name)}">Open change details →</a></p>
    </li>
  `;
}

function compareChangesForTriage(left: ViewerChange, right: ViewerChange): number {
  return triageScore(right) - triageScore(left) || left.name.localeCompare(right.name);
}

function triageScore(change: ViewerChange): number {
  const missingArtifacts = [
    change.hasProposal,
    change.hasDesign,
    change.hasTasks,
    change.hasSpecDeltas,
  ].filter((present) => !present).length;
  const incompleteTaskSignal = change.taskCounts.incomplete > 0 ? 1 : 0;
  return change.warnings.length * 100 + missingArtifacts * 10 + incompleteTaskSignal;
}

function renderDashboardSpecOverview(model: ViewerModel): string {
  const recentSpecs = selectRecentDashboardSpecs(model.specs);
  return html`
    <section class="dashboard-section">
      <div class="section-heading">
        <div>
          <p class="eyebrow">Recent spec preview</p>
          <h2>Current Specs</h2>
        </div>
        <a href="/specs">View all specs (${model.specs.length}) →</a>
      </div>
      ${
        model.specs.length === 0
          ? html`<p>No current specs found.</p>`
          : html`<p class="muted">Showing ${recentSpecs.length} of ${model.specs.length} current spec(s).</p>
              <ul class="dashboard-list">
                ${recentSpecs.map(renderDashboardSpec).join("\n")}
              </ul>`
      }
    </section>
  `;
}

function renderDashboardSpec(spec: ViewerSpec): string {
  return html`
    <li>
      <h3><a href="/specs/${encodeURIComponent(spec.name)}">${escapeHtml(spec.name)}</a> ${warningBadge(spec.warnings)}</h3>
      <p class="muted">${escapeHtml(spec.relativePath)}</p>
      ${spec.purpose ? html`<p>${escapeHtml(spec.purpose)}</p>` : "<p><em>No purpose found.</em></p>"}
      <p>${spec.requirementCount} requirement(s), ${spec.scenarioCount} scenario(s), ${spec.warnings.length} warning(s)</p>
    </li>
  `;
}

function renderDashboardWarningSummary(model: ViewerModel): string {
  return html`
    <section class="dashboard-section ${model.warnings.length > 0 ? "warnings" : ""}">
      <div class="section-heading">
        <div>
          <p class="eyebrow">Warning summary</p>
          <h2>Warnings</h2>
        </div>
        <a href="/warnings">View all warnings (${model.warnings.length}) →</a>
      </div>
      ${
        model.warnings.length === 0
          ? html`<p>No warnings found. Project health checks are clear.</p>`
          : html`<p>${model.warnings.length} warning(s) found.</p>
              <ul class="warning-list compact-list">
                ${model.warnings.slice(0, 5).map(renderWarningListItem).join("\n")}
              </ul>`
      }
    </section>
  `;
}

function renderDashboardArchiveSummary(model: ViewerModel): string {
  const recentArchivedChanges = selectRecentDashboardArchivedChanges(model.archivedChanges);
  return html`
    <section class="dashboard-section">
      <div class="section-heading">
        <div>
          <p class="eyebrow">Recent archive preview</p>
          <h2>Archived Changes</h2>
        </div>
        <a href="/archive">View all archived changes (${model.archivedChanges.length}) →</a>
      </div>
      ${
        model.archivedChanges.length === 0
          ? html`<p>No archived changes found.</p>`
          : html`<p class="muted">Showing ${recentArchivedChanges.length} of ${model.archivedChanges.length} archived change(s).</p>
              <ul class="dashboard-list compact-list">
                ${recentArchivedChanges.map(renderDashboardArchivedChange).join("\n")}
              </ul>`
      }
    </section>
  `;
}

function renderDashboardArchivedChange(change: ViewerChange): string {
  return html`
    <li>
      <h3><a href="/archive/${encodeURIComponent(change.name)}">${escapeHtml(change.name)}</a></h3>
      <p class="muted">${escapeHtml(change.relativePath)}</p>
      <p>Tasks: ${formatTaskProgress(change)}</p>
    </li>
  `;
}

function renderDashboardSupportMetadata(model: ViewerModel): string {
  return html`
    <section class="dashboard-section supporting-metadata">
      <div class="section-heading">
        <div>
          <p class="eyebrow">Project verification</p>
          <h2>Health & Paths</h2>
        </div>
      </div>
      <p>${projectHealthSummary(model)}</p>
      <p>
        <a href="/warnings">Warnings page (${model.summary.warnings}) →</a>
      </p>
      <dl class="metadata-list">
        <div><dt>Root</dt><dd>${escapeHtml(model.rootPath)}</dd></div>
        <div><dt>Config</dt><dd>${escapeHtml(model.configPath)}</dd></div>
      </dl>
    </section>
  `;
}

function selectRecentDashboardSpecs(specs: ViewerSpec[]): ViewerSpec[] {
  return [...specs].sort(compareSpecsByDeterministicRecency).slice(0, DASHBOARD_PREVIEW_LIMIT);
}

function selectRecentDashboardArchivedChanges(changes: ViewerChange[]): ViewerChange[] {
  return [...changes].sort(compareChangesByDeterministicRecency).slice(0, DASHBOARD_PREVIEW_LIMIT);
}

function compareSpecsByDeterministicRecency(left: ViewerSpec, right: ViewerSpec): number {
  return right.relativePath.localeCompare(left.relativePath) || right.name.localeCompare(left.name);
}

function compareChangesByDeterministicRecency(left: ViewerChange, right: ViewerChange): number {
  return right.name.localeCompare(left.name) || right.relativePath.localeCompare(left.relativePath);
}

function renderSpecsIndex(model: ViewerModel): string {
  return layout(
    "Specs",
    model,
    html`
      <h1>Current Specs</h1>
      ${
        model.specs.length === 0
          ? "<p>No current specs found.</p>"
          : html`<ul class="item-list">
            ${model.specs.map(renderSpecListItem).join("\n")}
          </ul>`
      }
    `,
  );
}

function renderSpecListItem(spec: ViewerSpec): string {
  return html`
    <li>
      <h2><a href="/specs/${encodeURIComponent(spec.name)}">${escapeHtml(spec.name)}</a> ${warningBadge(spec.warnings)}</h2>
      <p class="muted">${escapeHtml(spec.relativePath)}</p>
      ${spec.purpose ? html`<p>${escapeHtml(spec.purpose)}</p>` : "<p><em>No purpose found.</em></p>"}
      <p>${spec.requirementCount} requirement(s), ${spec.scenarioCount} scenario(s)</p>
    </li>
  `;
}

function renderSpecDetail(model: ViewerModel, spec: ViewerSpec): string {
  return layout(
    spec.name,
    model,
    html`
      <p><a href="/specs">← Specs</a></p>
      <h1>${escapeHtml(spec.name)} ${warningBadge(spec.warnings)}</h1>
      <p class="muted">${escapeHtml(spec.relativePath)}</p>
      <section>
        <h2>Summary</h2>
        ${spec.purpose ? html`<p>${escapeHtml(spec.purpose)}</p>` : "<p><em>No purpose found.</em></p>"}
        <p>${spec.requirementCount} requirement(s), ${spec.scenarioCount} scenario(s)</p>
      </section>
      ${renderWarningsBlock(spec.warnings)}
      <section>
        <h2>Requirements</h2>
        ${
          spec.requirements.length === 0
            ? "<p>No requirements found.</p>"
            : html`<ul>
              ${spec.requirements
                .map(
                  (requirement) => html`
                    <li>
                      <strong>${escapeHtml(requirement.title)}</strong>
                      <span class="muted">line ${requirement.line}</span>
                      <ul>
                        ${requirement.scenarios
                          .map(
                            (scenario) =>
                              html`<li>${escapeHtml(scenario.title)} <span class="muted">line ${scenario.line}</span></li>`,
                          )
                          .join("\n")}
                      </ul>
                    </li>
                  `,
                )
                .join("\n")}
            </ul>`
        }
      </section>
      <section>
        <h2>Markdown</h2>
        ${renderMarkdown(spec.markdown)}
      </section>
    `,
  );
}

function renderChangesIndex(model: ViewerModel): string {
  return layout(
    "Active Changes",
    model,
    html`
      <h1>Active Changes</h1>
      ${
        model.activeChanges.length === 0
          ? "<p>No active changes found.</p>"
          : html`<ul class="item-list">
            ${model.activeChanges.map((change) => renderChangeListItem(change, "/changes")).join("\n")}
          </ul>`
      }
    `,
  );
}

function renderArchiveIndex(model: ViewerModel): string {
  return layout(
    "Archived Changes",
    model,
    html`
      <h1>Archived Changes</h1>
      ${
        model.archivedChanges.length === 0
          ? "<p>No archived changes found.</p>"
          : html`<ul class="item-list">
            ${model.archivedChanges.map((change) => renderChangeListItem(change, "/archive")).join("\n")}
          </ul>`
      }
    `,
  );
}

function renderChangeListItem(change: ViewerChange, basePath: string): string {
  const active = !change.archived;
  return html`
    <li>
      <h2><a href="${basePath}/${encodeURIComponent(change.name)}">${escapeHtml(change.name)}</a> ${warningBadge(change.warnings)}</h2>
      <p class="muted">${escapeHtml(change.relativePath)}</p>
      ${active ? html`${renderPhaseBadge(change.lifecycle)}<p><strong>Next action:</strong> ${escapeHtml(primaryActionSummary(change.lifecycle))}</p>` : ""}
      <p>${artifactFlags(change)}</p>
      <p>Tasks: ${change.taskCounts.complete}/${change.taskCounts.total} complete</p>
      ${change.archived ? html`<p class="muted">Lifecycle complete: archived change.</p>` : ""}
    </li>
  `;
}

function renderChangeDetail(model: ViewerModel, change: ViewerChange): string {
  const backPath = change.archived ? "/archive" : "/changes";
  return layout(
    change.name,
    model,
    html`
      <p><a href="${backPath}">← ${change.archived ? "Archived changes" : "Active changes"}</a></p>
      <h1>${escapeHtml(change.name)} ${warningBadge(change.warnings)}</h1>
      <p class="muted">${escapeHtml(change.relativePath)}</p>
      ${
        change.archived
          ? renderArchivedLifecycleCompletion(change)
          : html`<p>
              <a class="button-link" href="/changes/${encodeURIComponent(change.name)}/review">Review this active change →</a>
              ${
                model.reviewCompanionEnabled
                  ? html`<a class="button-link" href="/changes/${encodeURIComponent(change.name)}/companion">Open review companion →</a>`
                  : ""
              }
            </p>
            ${renderLifecyclePanel(change.lifecycle, model.reviewCompanionEnabled, change.name)}`
      }
      <section>
        <h2>Artifacts</h2>
        ${renderArtifactCompletionDetails(change)}
      </section>
      ${renderWarningsBlock(change.warnings)}
      ${
        change.artifacts.length === 0
          ? "<p>No renderable Markdown artifacts found.</p>"
          : change.artifacts.map(renderArtifact).join("\n")
      }
    `,
  );
}

function renderChangeReview(model: ViewerModel, change: ViewerChange): string {
  return layout(
    `Review ${change.name}`,
    model,
    html`
      <p><a href="/changes/${encodeURIComponent(change.name)}">← Active change detail</a></p>
      <section class="hero">
        <p class="eyebrow">Pi-assisted review</p>
        <h1>Review ${escapeHtml(change.name)} ${warningBadge(change.warnings)}</h1>
        <p class="muted">${escapeHtml(change.relativePath)}</p>
        <p>
          Use this page to inspect the generated OpenSpec proposal artifacts before implementation
          or archive.
        </p>
        ${
          model.reviewCompanionEnabled
            ? html`<p><a class="button-link" href="/changes/${encodeURIComponent(change.name)}/companion">Open review companion →</a></p>`
            : ""
        }
      </section>
      ${renderLifecyclePanel(change.lifecycle, model.reviewCompanionEnabled, change.name)}
      <section class="notice">
        <h2>Feedback path</h2>
        ${
          model.reviewCompanionEnabled
            ? html`<p>
                Browser-native comments are available from the review companion page. Feedback
                saved there is stored as sidecar review records only and does not directly edit
                OpenSpec artifacts.
              </p>`
            : html`<p>
                Review companion mode is disabled for this viewer session. Browser feedback writes
                are rejected, and this page intentionally does not include comment boxes, save
                buttons, or edit controls.
              </p>`
        }
      </section>
      <section>
        <h2>Artifact completion details</h2>
        ${renderArtifactCompletionDetails(change)}
      </section>
      ${renderWarningsBlock(change.warnings)}
      <section>
        <h2>Review artifacts</h2>
        ${
          change.artifacts.length === 0
            ? "<p>No renderable Markdown artifacts found.</p>"
            : change.artifacts.map(renderArtifact).join("\n")
        }
      </section>
    `,
  );
}

function renderChangeCompanion(model: ViewerModel, change: ViewerChange): string {
  const reviewAgentPanel = renderReviewAgentPanel(model, change);
  return layout(
    `Review Companion ${change.name}`,
    model,
    html`
      <p><a href="/changes/${encodeURIComponent(change.name)}/review">← Active change review</a></p>
      <div class="companion-shell">
        <section class="hero companion-hero">
          <p class="eyebrow">Document-first review companion</p>
          <h1>Review Companion: ${escapeHtml(change.name)} ${warningBadge(change.warnings)}</h1>
          <p class="muted">${escapeHtml(change.relativePath)}</p>
          <p>
            Read the OpenSpec artifacts first. Select text to reveal a small comment action, then
            intentionally open a popover composer when you want to save sidecar feedback.
          </p>
          <p>
            Saved comments are feedback records only; they do not directly edit proposal, design,
            spec, or task artifacts.
          </p>
        </section>
        ${renderCompanionLifecycleGuidance(change, model.reviewCompanionEnabled)}
        <section class="notice companion-guidance" aria-label="Review guidance">
          <div>
            <p class="eyebrow">Review guidance</p>
            <h2>How to annotate</h2>
          </div>
          <p>
            Select relevant artifact text and use the contextual comment button. Existing exact and
            fuzzy comments appear as subtle markers in the document; all comments remain inspectable
            in the overview below.
          </p>
        </section>
        ${reviewAgentPanel}
        <section class="companion-document" aria-label="OpenSpec review document">
          <div class="section-heading">
            <div>
              <p class="eyebrow">Review stages</p>
              <h2>OpenSpec artifacts</h2>
            </div>
          </div>
          ${renderCompanionStageTabs(change)}
        </section>
        <div id="annotation-affordance" class="annotation-affordance" hidden>
          <button type="button" data-open-composer>Add comment</button>
        </div>
        <section id="comment-composer" class="comment-composer" aria-label="Selected text comment composer" hidden>
          <form id="feedback-form" data-change="${escapeHtml(change.name)}">
            <div class="section-heading compact-heading">
              <div>
                <p class="eyebrow">New sidecar comment</p>
                <h2>Comment on selected text</h2>
              </div>
              <button type="button" class="secondary-button" data-close-composer>Close</button>
            </div>
            <p class="muted" data-composer-context>Select artifact text, then choose Add comment.</p>
            <input name="artifactPath" type="hidden" required />
            <input name="selectedText" type="hidden" required />
            <input name="beforeText" type="hidden" />
            <input name="afterText" type="hidden" />
            <input name="startOffset" type="hidden" />
            <input name="endOffset" type="hidden" />
            <label>
              Category
              <input name="category" type="text" placeholder="clarity, completeness, risk" />
            </label>
            <label>
              Severity
              <select name="severity">
                <option value="suggestion">Suggestion</option>
                <option value="concern">Concern</option>
                <option value="blocking">Blocking</option>
              </select>
            </label>
            <label>
              Comment
              <textarea name="body" rows="5" required></textarea>
            </label>
            <button type="submit">Save feedback record</button>
          </form>
        </section>
        <section class="comments-overview" aria-labelledby="comments-overview-heading">
          <div class="section-heading">
            <div>
              <p class="eyebrow">Comments overview</p>
              <h2 id="comments-overview-heading">Feedback triage</h2>
            </div>
            <p id="feedback-status" class="muted" role="status"></p>
          </div>
          <div id="feedback-list" data-change="${escapeHtml(change.name)}" data-comments-overview>
            <p class="muted">Loading review companion feedback…</p>
          </div>
        </section>
        <div id="thread-popover" class="thread-popover" role="dialog" aria-label="Comment thread" hidden></div>
      </div>
      <script>${companionScript(change.name)}</script>
    `,
    { mainClass: "companion-page" },
  );
}

function renderReviewAgentPanel(model: ViewerModel, change: ViewerChange): string {
  if (model.reviewAgent.mode === "disabled") {
    return html`
      <section class="notice review-agent-panel disabled" aria-label="Pi review analysis">
        <p class="eyebrow">Ask Pi</p>
        <h2>Pi review analysis disabled</h2>
        <p>${escapeHtml(model.reviewAgent.reason ?? "Browser Pi review analysis is disabled for this serve session.")}</p>
        <p class="muted">Sidecar comment creation and thread review remain available.</p>
      </section>
    `;
  }

  if (model.reviewAgent.mode === "unavailable") {
    return html`
      <section class="notice review-agent-panel unavailable" aria-label="Pi review analysis">
        <p class="eyebrow">Ask Pi</p>
        <h2>Pi review analysis unavailable</h2>
        <p>${escapeHtml(model.reviewAgent.reason ?? "Pi SDK setup, credentials, or model availability prevented live analysis.")}</p>
        <p class="muted">Sidecar comment creation and thread review remain available.</p>
      </section>
    `;
  }

  return html`
    <section class="notice review-agent-panel" aria-label="Pi review analysis">
      <div class="section-heading compact-heading">
        <div>
          <p class="eyebrow">Ask Pi</p>
          <h2>Live review analysis</h2>
        </div>
        <button type="button" data-review-agent-start data-change="${escapeHtml(change.name)}">Ask Pi</button>
      </div>
      <p>
        Browser Pi review analysis is enabled by default. Sessions are local, active-change-only,
        and non-mutating; Pi can suggest updates but this page will not apply artifact edits.
      </p>
      <details class="review-agent-context" open>
        <summary>Context that will be sent to Pi</summary>
        <ul>
          <li>Open or selected sidecar feedback threads from this active change.</li>
          <li>${change.artifacts.length} artifact snapshot(s): ${escapeHtml(change.artifacts.map((artifact) => artifact.relativePath).join(", ") || "none")}</li>
          <li>${change.warnings.length} analyzer warning(s) associated with this change.</li>
        </ul>
      </details>
      <div id="review-agent-transcript" class="review-agent-transcript" data-review-agent-transcript>
        <p class="muted">Pi analysis transcript will appear here.</p>
      </div>
      <form id="review-agent-follow-up" data-review-agent-follow-up hidden>
        <label>
          Follow-up instruction
          <textarea name="message" rows="3" required></textarea>
        </label>
        <button type="submit">Send follow-up</button>
      </form>
    </section>
  `;
}

function renderLifecyclePanel(
  guidance: LifecycleGuidance,
  companionEnabled: boolean,
  changeName?: string,
): string {
  return html`
    <section class="notice lifecycle-panel" aria-label="Lifecycle guidance">
      <div class="section-heading compact-heading">
        <div>
          <p class="eyebrow">Lifecycle guidance</p>
          <h2>${renderPhaseBadge(guidance)} ${escapeHtml(guidance.phase.label)} phase</h2>
        </div>
      </div>
      <p>${escapeHtml(guidance.summary)}</p>
      <h3>Why this phase?</h3>
      <ul>
        ${guidance.reasons.map((reason) => html`<li>${escapeHtml(reason)}</li>`).join("\n")}
      </ul>
      <h3>Recommended next actions</h3>
      ${renderLifecycleActions(guidance.actions, companionEnabled, changeName)}
      ${renderLifecycleSafetyNotes(guidance.safetyNotes)}
    </section>
  `;
}

function renderArchivedLifecycleCompletion(change: ViewerChange): string {
  return html`
    <section class="notice lifecycle-panel" aria-label="Lifecycle completion">
      <p class="eyebrow">Lifecycle complete</p>
      <h2>${escapeHtml(change.name)} is archived</h2>
      <p>This archived change is retained for review history. Active Propose, Review, Apply, and Archive next-action guidance is not shown for archived changes.</p>
    </section>
  `;
}

function renderCompanionLifecycleGuidance(change: ViewerChange, companionEnabled: boolean): string {
  const phase = change.lifecycle.phase.id;
  const phaseCopy =
    phase === "review"
      ? "Read proposal, design, tasks, and spec deltas; use sidecar comments or Pi analysis to capture feedback before implementation."
      : phase === "apply"
        ? "Comments and Pi analysis remain guidance; explicitly apply approved artifact or code updates through Pi or CLI workflows."
        : phase === "archive"
          ? "Validate the completed change, then archive with an explicit CLI command when ready."
          : "Refine missing proposal artifacts before relying on review or implementation guidance.";

  return html`
    <section class="notice lifecycle-panel companion-lifecycle" aria-label="Companion lifecycle guidance">
      <p class="eyebrow">Lifecycle-aware companion guidance</p>
      <h2>${renderPhaseBadge(change.lifecycle)} ${escapeHtml(change.lifecycle.phase.label)} phase</h2>
      <p>${escapeHtml(change.lifecycle.summary)}</p>
      <p>${escapeHtml(phaseCopy)}</p>
      ${renderLifecycleActions(change.lifecycle.actions, companionEnabled, change.name)}
      ${renderLifecycleSafetyNotes(change.lifecycle.safetyNotes)}
    </section>
  `;
}

function renderLifecycleActions(
  actions: LifecycleAction[],
  companionEnabled: boolean,
  changeName?: string,
): string {
  if (actions.length === 0) {
    return html`<p class="muted">No active lifecycle action is recommended for this archived state.</p>`;
  }

  return html`
    <ul class="lifecycle-actions">
      ${actions
        .filter((action) => companionEnabled || !action.href?.endsWith("/companion"))
        .map(
          (action) => html`
            <li>
              <strong>${escapeHtml(action.label)}</strong>
              <span>${escapeHtml(action.summary)}</span>
              ${action.href ? html`<a href="${action.href}">Open →</a>` : ""}
              ${action.command ? html`<code>${escapeHtml(action.command)}</code>` : ""}
              ${renderLifecycleActionButton(action, changeName)}
            </li>
          `,
        )
        .join("\n")}
    </ul>
  `;
}

function renderLifecycleActionButton(action: LifecycleAction, changeName?: string): string {
  const kind = lifecycleActionKind(action, changeName);
  if (!kind) return "";

  const payload = { kind, ...(changeName ? { changeName } : {}) };
  const mutating = kind === "validate" || kind === "archive";
  return html`
    <button
      type="button"
      class="secondary-button lifecycle-action-button"
      data-lifecycle-action="${escapeHtml(JSON.stringify(payload))}"
      data-lifecycle-confirm="${mutating ? escapeHtml(`Run ${action.command} locally?`) : ""}"
    >
      Run local action
    </button>
  `;
}

function lifecycleActionKind(action: LifecycleAction, changeName?: string): string | undefined {
  if (action.command === "openspec list") return "explore";
  if (action.command?.startsWith("Ask Pi: explore")) return "explore";
  if (action.command?.startsWith("Ask Pi: propose or continue")) return "proposal-handoff";
  if (action.command?.startsWith("Ask Pi: propose")) return "new-change-handoff";
  if (action.command?.startsWith("/opsx-apply") && changeName) return "apply-handoff";
  if (action.command?.startsWith("openspec validate") && changeName) return "validate";
  if (action.command?.startsWith("openspec archive") && changeName) return "archive";
  return undefined;
}

function renderLifecycleSafetyNotes(notes: string[]): string {
  if (notes.length === 0) return "";

  return html`
    <div class="lifecycle-safety">
      <strong>Safety:</strong>
      <ul>
        ${notes.map((note) => html`<li>${escapeHtml(note)}</li>`).join("\n")}
      </ul>
    </div>
  `;
}

function renderPhaseBadge(guidance: LifecycleGuidance): string {
  return html`<span class="phase-badge phase-${guidance.phase.id}">${escapeHtml(guidance.phase.label)}</span>`;
}

function primaryActionSummary(guidance: LifecycleGuidance): string {
  return guidance.actions[0]?.summary ?? guidance.summary;
}

function renderArtifact(artifact: ViewerArtifact): string {
  return html`
    <section>
      <h2>${escapeHtml(artifact.name)}</h2>
      <p class="muted">${escapeHtml(artifact.relativePath)}</p>
      ${renderMarkdown(artifact.markdown)}
    </section>
  `;
}

function renderCompanionStageTabs(change: ViewerChange): string {
  const grouped = groupCompanionArtifacts(change.artifacts);
  return html`
    <div class="companion-tabs" data-companion-tabs>
      <div class="tab-list" role="tablist" aria-label="Review stages">
        ${COMPANION_STAGES.map(
          (stage, index) => html`
            <button
              type="button"
              class="tab-button ${index === 0 ? "active" : ""}"
              role="tab"
              aria-selected="${index === 0 ? "true" : "false"}"
              aria-controls="companion-stage-${stage.id}"
              id="companion-tab-${stage.id}"
              data-stage-tab="${stage.id}"
            >
              ${escapeHtml(stage.label)} <span class="nav-count">${grouped[stage.id].length}</span>
            </button>
          `,
        ).join("\n")}
      </div>
      ${COMPANION_STAGES.map(
        (stage, index) => html`
          <section
            class="tab-panel"
            role="tabpanel"
            id="companion-stage-${stage.id}"
            aria-labelledby="companion-tab-${stage.id}"
            data-stage-panel="${stage.id}"
            ${index === 0 ? "" : "hidden"}
          >
            <h3>${escapeHtml(stage.label)}</h3>
            ${
              grouped[stage.id].length === 0
                ? html`<p class="muted">No ${escapeHtml(stage.label.toLowerCase())} artifact is available for this change.</p>`
                : grouped[stage.id].map(renderCompanionArtifact).join("\n")
            }
          </section>
        `,
      ).join("\n")}
    </div>
  `;
}

function groupCompanionArtifacts(
  artifacts: ViewerArtifact[],
): Record<CompanionStageId, ViewerArtifact[]> {
  const grouped: Record<CompanionStageId, ViewerArtifact[]> = {
    proposal: [],
    design: [],
    tasks: [],
    "spec-deltas": [],
  };

  for (const artifact of artifacts) {
    if (artifact.name === "Proposal") grouped.proposal.push(artifact);
    else if (artifact.name === "Design") grouped.design.push(artifact);
    else if (artifact.name === "Tasks") grouped.tasks.push(artifact);
    else grouped["spec-deltas"].push(artifact);
  }

  return grouped;
}

function renderCompanionArtifact(artifact: ViewerArtifact): string {
  return html`
    <section class="companion-artifact" data-artifact-path="${escapeHtml(artifact.relativePath)}">
      <header class="artifact-header">
        <div>
          <h4>${escapeHtml(artifact.name)}</h4>
          <p class="muted">${escapeHtml(artifact.relativePath)}</p>
        </div>
        <p class="selection-hint">Select text to add a contextual sidecar comment.</p>
      </header>
      <div class="artifact-content" data-artifact-markdown="${escapeHtml(artifact.markdown)}" data-marker-host>
        ${renderMarkdown(artifact.markdown)}
      </div>
    </section>
  `;
}

function renderArtifactCompletionDetails(change: ViewerChange): string {
  return html`
    <p>${artifactFlags(change)}</p>
    <dl class="artifact-details">
      <div><dt>Proposal</dt><dd>${change.hasProposal ? "present" : "missing"}</dd></div>
      <div><dt>Design</dt><dd>${change.hasDesign ? "present" : "missing"}</dd></div>
      <div><dt>Tasks</dt><dd>${change.hasTasks ? `${change.taskCounts.complete}/${change.taskCounts.total} complete` : "missing"}</dd></div>
      <div><dt>Spec deltas</dt><dd>${change.hasSpecDeltas ? "present" : "missing"}</dd></div>
    </dl>
    ${renderTaskActionList(change)}
  `;
}

function renderTaskActionList(change: ViewerChange): string {
  if (change.archived || change.taskItems.length === 0) return "";

  return html`
    <div class="task-action-list" aria-label="Task state actions">
      <h3>Task state actions</h3>
      <ul>
        ${change.taskItems
          .map((task) => {
            const payload = {
              kind: "mark-task",
              changeName: change.name,
              line: task.line,
              checked: !task.checked,
            };
            return html`
              <li>
                <span>${task.checked ? "✓" : "—"} line ${task.line}: ${escapeHtml(task.text)}</span>
                <button
                  type="button"
                  class="secondary-button lifecycle-action-button"
                  data-lifecycle-action="${escapeHtml(JSON.stringify(payload))}"
                  data-lifecycle-confirm="${escapeHtml(`Mark task on line ${task.line} ${task.checked ? "incomplete" : "complete"}?`)}"
                >
                  Mark ${task.checked ? "incomplete" : "complete"}
                </button>
              </li>
            `;
          })
          .join("\n")}
      </ul>
    </div>
  `;
}

function renderWarnings(model: ViewerModel): string {
  return layout(
    "Warnings",
    model,
    html`
      <h1>Warnings</h1>
      ${
        model.warnings.length === 0
          ? "<p>No warnings found.</p>"
          : html`<ul class="warning-list">
            ${model.warnings.map(renderWarningListItem).join("\n")}
          </ul>`
      }
    `,
  );
}

function renderRefreshError(model: ViewerModel): string {
  if (!model.refreshError) {
    return "";
  }

  return html`
    <section class="warnings" role="alert">
      <h2>Latest watch refresh failed</h2>
      <p>${escapeHtml(model.refreshError)}</p>
      <p class="muted">Serving the last successfully built OpenSpec model.</p>
    </section>
  `;
}

function renderWarningsBlock(warnings: ProjectWarning[]): string {
  if (warnings.length === 0) {
    return "";
  }

  return html`
    <section class="warnings">
      <h2>Warnings</h2>
      <ul>
        ${warnings
          .map(
            (warning) =>
              html`<li><strong>${escapeHtml(warning.code)}</strong>: ${escapeHtml(warning.message)}</li>`,
          )
          .join("\n")}
      </ul>
    </section>
  `;
}

function layout(
  title: string,
  model: ViewerModel,
  body: string,
  options: { mainClass?: string } = {},
): string {
  return html`<!doctype html>
    <html lang="en">
      <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <title>${escapeHtml(title)} - openspec-doc</title>
        <style>${styles()}</style>
      </head>
      <body>
        <header class="site-header">
          <a class="brand" href="/">openspec-doc</a>
          ${renderPersistentNavigation(model)}
        </header>
        <main${options.mainClass ? html` class="${escapeHtml(options.mainClass)}"` : ""}>
          ${renderRefreshError(model)}
          ${body}
        </main>
        <script>${lifecycleActionScript()}</script>
        ${model.watchEnabled ? html`<script>${watchReloadScript(model.modelVersion)}</script>` : ""}
      </body>
    </html>`;
}

function renderPersistentNavigation(model: ViewerModel): string {
  return html`
    <nav class="primary-nav" aria-label="Viewer sections">
      <a href="/">Dashboard</a>
      <a href="/specs">Specs <span class="nav-count">${model.summary.currentSpecs}</span></a>
      <a href="/changes">Active Changes <span class="nav-count">${model.summary.activeChanges}</span></a>
      <a href="/archive">Archive <span class="nav-count">${model.summary.archivedChanges}</span></a>
      <a href="/warnings">Warnings <span class="nav-count">${model.summary.warnings}</span></a>
    </nav>
  `;
}

function renderMarkdown(markdown: string): string {
  const blocks = markdown
    .split(/\n{2,}/)
    .map((block) => block.trim())
    .filter(Boolean);
  return html`<div class="markdown">
    ${blocks.map(renderMarkdownBlock).join("\n")}
  </div>`;
}

function renderMarkdownBlock(block: string): string {
  const heading = block.match(/^(#{1,6})\s+(.+)$/);
  if (heading) {
    const level = heading[1].length;
    return `<h${level}>${escapeHtml(heading[2])}</h${level}>`;
  }

  if (block.startsWith("- ")) {
    return html`<ul>
      ${block
        .split(/\r?\n/)
        .filter((line) => line.startsWith("- "))
        .map((line) => html`<li>${escapeHtml(line.slice(2))}</li>`)
        .join("\n")}
    </ul>`;
  }

  return html`<p>${escapeHtml(block).replaceAll("\n", "<br />")}</p>`;
}

function countCard(label: string, value: number, href?: string): string {
  const content = html`<span class="count">${value}</span><span>${escapeHtml(label)}</span>`;
  return href
    ? html`<a class="card" href="${href}">${content}</a>`
    : html`<div class="card">${content}</div>`;
}

function renderWarningListItem(warning: ProjectWarning): string {
  return html`
    <li>
      <strong>${escapeHtml(warning.code)}</strong>
      <span class="muted">${escapeHtml(warning.path)}</span><br />
      ${escapeHtml(warning.message)}
    </li>
  `;
}

function formatTaskProgress(change: ViewerChange): string {
  return change.taskCounts.total === 0
    ? "no tasks found"
    : `${change.taskCounts.complete}/${change.taskCounts.total} complete`;
}

function artifactFlags(change: ViewerChange): string {
  return [
    flag("proposal", change.hasProposal),
    flag("design", change.hasDesign),
    flag("tasks", change.hasTasks),
    flag("spec deltas", change.hasSpecDeltas),
  ].join(" ");
}

function flag(label: string, present: boolean): string {
  return html`<span class="flag ${present ? "present" : "missing"}">${present ? "✓" : "—"} ${escapeHtml(label)}</span>`;
}

function warningBadge(warnings: ProjectWarning[]): string {
  return warnings.length > 0
    ? html`<a class="badge" href="/warnings">${warnings.length} warning(s)</a>`
    : "";
}

function ok(html: string): RenderedPage {
  return { status: 200, html };
}

function notFound(model: ViewerModel, message: string): RenderedPage {
  return {
    status: 404,
    html: layout("Not Found", model, html`<h1>Not Found</h1><p>${escapeHtml(message)}</p>`),
  };
}

function normalizeRoute(routePath: string): string {
  const normalized = routePath.replace(/\/+$/, "");
  return normalized === "" ? "/" : normalized;
}

function escapeHtml(value: string | number): string {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function html(strings: TemplateStringsArray, ...values: unknown[]): string {
  return strings.reduce((output, string, index) => `${output}${string}${values[index] ?? ""}`, "");
}

function styles(): string {
  return `
    :root { color-scheme: light dark; font-family: Inter, ui-sans-serif, system-ui, sans-serif; line-height: 1.5; }
    body { margin: 0; background: #0f172a; color: #f9fafb; }
    .site-header { display: flex; gap: 1.5rem; align-items: center; padding: 1rem 2rem; background: #020617; border-bottom: 1px solid #334155; position: sticky; top: 0; z-index: 10; }
    main { max-width: 1120px; margin: 0 auto; padding: 2.25rem; }
    a { color: #93c5fd; text-decoration: none; } a:hover { text-decoration: underline; }
    .brand { font-weight: 800; color: #f9fafb; white-space: nowrap; }
    .primary-nav { display: flex; gap: 0.5rem; flex-wrap: wrap; align-items: center; }
    .primary-nav a { border: 1px solid #334155; border-radius: 999px; color: #dbeafe; padding: 0.35rem 0.7rem; }
    .primary-nav a:hover { background: #1e293b; text-decoration: none; }
    .nav-count { display: inline-block; min-width: 1.35rem; border-radius: 999px; background: #1d4ed8; color: #eff6ff; font-size: 0.8rem; font-weight: 800; margin-left: 0.25rem; padding: 0 0.35rem; text-align: center; }
    .hero, section, .item-list > li { background: #1e293b; border: 1px solid #334155; border-radius: 1rem; padding: 1.15rem; margin: 1rem 0; }
    .notice { border-color: #60a5fa; }
    .eyebrow { color: #bfdbfe; font-size: 0.8rem; font-weight: 700; letter-spacing: 0.08em; text-transform: uppercase; }
    .button-link { display: inline-block; background: #2563eb; color: #eff6ff; border-radius: 0.5rem; padding: 0.5rem 0.75rem; font-weight: 700; }
    .lifecycle-overview { background: #111d3a; border-color: #3b82f6; margin-bottom: 1rem; }
    .lifecycle-phases { display: grid; grid-template-columns: repeat(auto-fit, minmax(170px, 1fr)); gap: 0.6rem; list-style: none; padding: 0; }
    .lifecycle-phases li, .lifecycle-actions li { background: rgba(15, 23, 42, 0.65); border: 1px solid #334155; border-radius: 0.65rem; padding: 0.65rem; }
    .lifecycle-phases span:last-child, .lifecycle-actions span { display: block; margin-top: 0.35rem; color: #cbd5e1; }
    .lifecycle-panel { background: #16213b; }
    .compact-panel { margin-top: 1rem; }
    .phase-badge { display: inline-block; border-radius: 999px; color: #eff6ff; font-size: 0.78rem; font-weight: 900; letter-spacing: 0.04em; padding: 0.18rem 0.55rem; text-transform: uppercase; vertical-align: middle; }
    .phase-explore { background: #0f766e; } .phase-propose { background: #7c3aed; } .phase-review { background: #2563eb; } .phase-apply { background: #b45309; } .phase-archive { background: #047857; }
    .lifecycle-actions { display: grid; gap: 0.55rem; list-style: none; padding: 0; }
    .lifecycle-actions code { display: block; margin-top: 0.45rem; overflow-x: auto; background: #020617; border: 1px solid #334155; border-radius: 0.45rem; padding: 0.45rem; }
    .lifecycle-action-button { margin-top: 0.5rem; }
    .lifecycle-action-result { background: #052e16; border: 1px solid #16a34a; border-radius: 0.65rem; margin-top: 0.85rem; padding: 0.75rem; }
    .lifecycle-action-result.failed { background: #450a0a; border-color: #dc2626; }
    .lifecycle-action-result pre { white-space: pre-wrap; overflow-x: auto; }
    .task-action-list { margin-top: 1rem; }
    .task-action-list ul { display: grid; gap: 0.5rem; list-style: none; padding: 0; }
    .task-action-list li { align-items: center; background: rgba(15, 23, 42, 0.45); border: 1px solid #334155; border-radius: 0.65rem; display: flex; gap: 0.75rem; justify-content: space-between; padding: 0.65rem; }
    .lifecycle-safety { background: rgba(14, 116, 144, 0.22); border: 1px solid #0e7490; border-radius: 0.65rem; margin-top: 0.85rem; padding: 0.75rem; }
    .lifecycle-safety ul { margin-bottom: 0; }
    .artifact-details { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 0.5rem; }
    .artifact-details div { background: #111827; border: 1px solid #374151; border-radius: 0.5rem; padding: 0.75rem; }
    .artifact-details dt { color: #9ca3af; } .artifact-details dd { margin: 0; font-weight: 700; }
    .cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 0.85rem; background: transparent; border: 0; padding: 0; }
    .card { display: flex; flex-direction: column; background: #1e293b; border: 1px solid #334155; border-radius: 0.9rem; padding: 0.95rem; color: #f9fafb; }
    .count { font-size: 2rem; font-weight: 800; line-height: 1; margin-bottom: 0.35rem; }
    .dashboard-hero { margin-bottom: 1rem; }
    .dashboard-counts { margin-bottom: 1.25rem; }
    .dashboard-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 1rem; align-items: start; margin-top: 1rem; }
    .dashboard-section { margin: 0; }
    .dashboard-primary-section { background: #172554; border-color: #2563eb; box-shadow: 0 18px 60px rgba(15, 23, 42, 0.35); }
    .secondary-dashboard-grid .dashboard-section { background: #182235; }
    .supporting-metadata { color: #cbd5e1; }
    .metadata-list { display: grid; gap: 0.65rem; margin: 0; }
    .metadata-list div { min-width: 0; }
    .metadata-list dt { color: #94a3b8; font-size: 0.8rem; font-weight: 800; letter-spacing: 0.06em; text-transform: uppercase; }
    .metadata-list dd { margin: 0.15rem 0 0; overflow-wrap: anywhere; }
    .section-heading { display: flex; justify-content: space-between; gap: 1rem; align-items: start; border-bottom: 1px solid #334155; margin-bottom: 1rem; padding-bottom: 0.75rem; }
    .section-heading h2, .section-heading p { margin: 0; }
    .section-heading a { font-weight: 700; white-space: nowrap; }
    .dashboard-list { list-style: none; padding: 0; margin: 0; display: grid; gap: 0.75rem; }
    .dashboard-list > li { background: rgba(15, 23, 42, 0.78); border: 1px solid #334155; border-radius: 0.75rem; padding: 0.85rem; }
    .dashboard-list h3 { margin-top: 0; }
    .compact-list > li { padding: 0.6rem 0.75rem; }
    .muted { color: #94a3b8; }
    .item-list, .warning-list { list-style: none; padding: 0; }
    .badge { display: inline-block; font-size: 0.8rem; background: #92400e; color: #fffbeb; border-radius: 999px; padding: 0.15rem 0.5rem; }
    .flag { display: inline-block; border-radius: 999px; padding: 0.15rem 0.5rem; margin: 0.1rem; background: #374151; }
    .flag.present { background: #065f46; } .flag.missing { background: #7f1d1d; }
    .warnings { border-color: #f59e0b; }
    .markdown { background: #111827; border: 1px solid #374151; border-radius: 0.5rem; padding: 1rem; overflow-x: auto; }
    main.companion-page { max-width: 1520px; }
    .companion-shell { display: grid; gap: 1rem; }
    .companion-guidance { display: grid; grid-template-columns: minmax(180px, 0.35fr) minmax(0, 1fr); gap: 1rem; align-items: start; }
    .companion-guidance h2, .companion-guidance p { margin-top: 0; }
    .companion-document { margin: 0; }
    .companion-tabs { display: grid; gap: 1rem; }
    .tab-list { display: flex; flex-wrap: wrap; gap: 0.5rem; position: sticky; top: 4.5rem; z-index: 5; background: rgba(30, 41, 59, 0.92); border: 1px solid #334155; border-radius: 0.75rem; padding: 0.5rem; backdrop-filter: blur(12px); }
    .tab-button.active { background: #1d4ed8; }
    .tab-panel { background: transparent; border: 0; margin: 0; padding: 0; }
    .companion-artifact { position: relative; max-width: 920px; margin-left: auto; margin-right: auto; }
    .artifact-header { display: flex; justify-content: space-between; gap: 1rem; align-items: start; }
    .artifact-header h4, .artifact-header p { margin-top: 0; }
    .companion-artifact .markdown { font-size: 1.02rem; line-height: 1.72; }
    .annotation-affordance { position: absolute; z-index: 30; filter: drop-shadow(0 12px 24px rgba(0,0,0,0.35)); }
    .comment-composer, .thread-popover { position: absolute; z-index: 40; width: min(420px, calc(100vw - 2rem)); background: #172033; border: 1px solid #60a5fa; border-radius: 0.9rem; box-shadow: 0 24px 80px rgba(2, 6, 23, 0.55); padding: 1rem; }
    .comment-composer.inline-fallback { position: static; width: auto; margin: 1rem 0; }
    .compact-heading { align-items: center; margin-bottom: 0.75rem; }
    .secondary-button { background: #334155; }
    .danger-button { background: #991b1b; color: #fee2e2; }
    .comment-marker { float: right; clear: right; min-width: 1.85rem; height: 1.85rem; margin: 0.15rem 0 0.35rem 0.6rem; padding: 0; border-radius: 999px; background: #2563eb; color: #eff6ff; font-size: 0.8rem; }
    .comment-marker.fuzzy { background: #7c3aed; }
    .comment-marker.emphasized { outline: 3px solid #fbbf24; transform: scale(1.15); }
    .artifact-marker-list { display: flex; gap: 0.35rem; justify-content: flex-end; }
    .comments-overview { margin-top: 1rem; }
    .feedback-group { background: rgba(15, 23, 42, 0.45); }
    label { display: grid; gap: 0.25rem; margin: 0.75rem 0; font-weight: 700; }
    input, select, textarea { width: 100%; box-sizing: border-box; border: 1px solid #4b5563; border-radius: 0.5rem; background: #111827; color: #f9fafb; padding: 0.5rem; font: inherit; }
    button { background: #2563eb; color: #eff6ff; border: 0; border-radius: 0.5rem; padding: 0.6rem 0.8rem; font-weight: 800; cursor: pointer; }
    .feedback-record { border-top: 1px solid #374151; padding-top: 0.75rem; margin-top: 0.75rem; }
    .selection-hint { color: #bfdbfe; font-size: 0.9rem; }
    .companion-artifact.has-selection { border-color: #93c5fd; box-shadow: 0 0 0 1px #93c5fd inset; }
    @media (max-width: 900px) { .dashboard-grid, .companion-guidance { grid-template-columns: 1fr; } .tab-list { position: static; } }
    @media (max-width: 640px) { .site-header { align-items: stretch; flex-direction: column; padding: 0.75rem 1rem; position: static; } main { padding: 1rem; } .primary-nav { display: grid; grid-template-columns: 1fr 1fr; } .primary-nav a { text-align: center; } .cards { grid-template-columns: repeat(2, minmax(0, 1fr)); } .section-heading, .artifact-header { flex-direction: column; } .section-heading a { white-space: normal; } }
    @media (max-width: 420px) { .cards, .primary-nav { grid-template-columns: 1fr; } }
  `;
}

function lifecycleActionScript(): string {
  return `
    (() => {
      function renderActionResult(payload, button) {
        const host = button.closest(".lifecycle-panel, .lifecycle-overview, section") || document.body;
        let result = host.querySelector("[data-lifecycle-action-result]");
        if (!result) {
          result = document.createElement("div");
          result.setAttribute("data-lifecycle-action-result", "");
          result.className = "lifecycle-action-result";
          host.append(result);
        }
        const body = payload.result || payload;
        const ok = body.ok !== false && !payload.error;
        result.classList.toggle("failed", !ok);
        const output = body.output || payload.error || "";
        result.innerHTML = "<p><strong>" + (ok ? "Action complete" : "Action failed") + ":</strong> "
          + escapeActionText(body.summary || payload.error || "Lifecycle action finished.") + "</p>"
          + (output ? "<pre>" + escapeActionText(output) + "</pre>" : "");
      }

      function escapeActionText(value) {
        return String(value).replace(/[&<>\"']/g, (character) => {
          if (character === "&") return "&amp;";
          if (character === "<") return "&lt;";
          if (character === ">") return "&gt;";
          if (character === "\\\"") return "&quot;";
          return "&#39;";
        });
      }

      document.addEventListener("click", async (event) => {
        const button = event.target.closest("[data-lifecycle-action]");
        if (!button) return;
        const confirmation = button.dataset.lifecycleConfirm;
        if (confirmation && !window.confirm(confirmation)) return;
        let payload;
        try {
          payload = JSON.parse(button.dataset.lifecycleAction || "{}");
        } catch {
          renderActionResult({ error: "Invalid lifecycle action payload." }, button);
          return;
        }
        button.disabled = true;
        try {
          const response = await fetch("/api/lifecycle-actions", {
            method: "POST",
            headers: { "content-type": "application/json" },
            body: JSON.stringify(payload),
          });
          const result = await response.json().catch(() => ({ error: "Lifecycle action returned invalid JSON." }));
          renderActionResult(result, button);
          if (response.ok && result.result?.refreshed) {
            window.setTimeout(() => window.location.reload(), 700);
          }
        } catch (error) {
          renderActionResult({ error: error instanceof Error ? error.message : "Lifecycle action failed." }, button);
        } finally {
          button.disabled = false;
        }
      });
    })();
  `;
}

function watchReloadScript(initialVersion: number): string {
  return `
    (() => {
      let currentVersion = ${JSON.stringify(initialVersion)};
      async function checkForRefresh() {
        try {
          const response = await fetch("/api/model-version", { cache: "no-store" });
          if (!response.ok) return;
          const payload = await response.json();
          if (typeof payload.version === "number" && payload.version > currentVersion) {
            window.location.reload();
            return;
          }
          if (typeof payload.version === "number") currentVersion = payload.version;
        } catch {
          // Ignore transient polling failures; the next tick can recover.
        }
      }
      window.setInterval(checkForRefresh, 1000);
    })();
  `;
}

function companionScript(changeName: string): string {
  return `
    const changeName = ${JSON.stringify(changeName)};
    const list = document.querySelector("#feedback-list");
    const form = document.querySelector("#feedback-form");
    const status = document.querySelector("#feedback-status");
    const affordance = document.querySelector("#annotation-affordance");
    const composer = document.querySelector("#comment-composer");
    const threadPopover = document.querySelector("#thread-popover");
    let pendingSelection = null;
    let loadedFeedback = [];
    let activeReviewAgentSession = null;
    let reviewAgentStreaming = false;

    function initializeTabs() {
      document.querySelectorAll("[data-stage-tab]").forEach((tab) => {
        tab.addEventListener("click", () => {
          const stage = tab.dataset.stageTab;
          document.querySelectorAll("[data-stage-tab]").forEach((item) => {
            const active = item === tab;
            item.classList.toggle("active", active);
            item.setAttribute("aria-selected", active ? "true" : "false");
          });
          document.querySelectorAll("[data-stage-panel]").forEach((panel) => {
            panel.hidden = panel.dataset.stagePanel !== stage;
          });
        });
      });
    }

    function initializeReviewAgent() {
      const startButton = document.querySelector("[data-review-agent-start]");
      const transcript = document.querySelector("[data-review-agent-transcript]");
      const followUpForm = document.querySelector("[data-review-agent-follow-up]");
      if (!startButton || !transcript) return;

      startButton.addEventListener("click", async () => {
        transcript.innerHTML = "<p class=\\\"muted\\\">Starting Pi review analysis…</p>";
        const response = await fetch("/api/review-agent/" + encodeURIComponent(changeName) + "/sessions", {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            reviewerInstruction: "Analyze this active OpenSpec change and included browser feedback.",
            selectedCommentIds: selectedFeedbackIds(),
            includeAllOpenFeedback: selectedFeedbackIds().length === 0,
          }),
        });
        const payload = await response.json().catch(() => ({}));
        if (!response.ok) {
          renderReviewAgentFailure(payload.error || "Pi review analysis could not start.", true);
          return;
        }
        activeReviewAgentSession = payload.session;
        reviewAgentStreaming = true;
        followUpForm.hidden = true;
        transcript.innerHTML = "<p class=\\\"muted\\\">Pi review analysis running…</p><pre data-review-agent-output></pre>";
        connectReviewAgentEvents(payload.eventsUrl);
      });

      followUpForm?.addEventListener("submit", async (event) => {
        event.preventDefault();
        if (!activeReviewAgentSession) return;
        if (reviewAgentStreaming) {
          status.textContent = "Wait for the current Pi response to finish before sending a follow-up.";
          return;
        }
        const data = Object.fromEntries(new FormData(followUpForm).entries());
        await fetch("/api/review-agent/sessions/" + encodeURIComponent(activeReviewAgentSession.id) + "/message", {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(data),
        });
        followUpForm.reset();
      });
    }

    function selectedFeedbackIds() {
      return Array.from(document.querySelectorAll("[data-feedback-select]:checked")).map((item) => item.value);
    }

    function connectReviewAgentEvents(eventsUrl) {
      const transcript = document.querySelector("[data-review-agent-transcript]");
      const output = transcript?.querySelector("[data-review-agent-output]");
      const source = new EventSource(eventsUrl);
      source.addEventListener("text_delta", (event) => {
        const payload = JSON.parse(event.data);
        output.textContent += payload.delta || "";
      });
      source.addEventListener("tool_start", (event) => {
        const payload = JSON.parse(event.data);
        transcript.insertAdjacentHTML("beforeend", "<p class=\\\"muted\\\">Using tool: " + escapeText(payload.toolName || "tool") + "</p>");
      });
      source.addEventListener("session_finished", () => {
        reviewAgentStreaming = false;
        document.querySelector("[data-review-agent-follow-up]").hidden = false;
        transcript.insertAdjacentHTML("beforeend", "<p><strong>Pi analysis complete.</strong></p>");
        source.close();
      });
      source.addEventListener("error", (event) => {
        reviewAgentStreaming = false;
        const payload = event.data ? JSON.parse(event.data) : { error: "Pi review event stream failed." };
        renderReviewAgentFailure(payload.error || "Pi review analysis failed.", Boolean(payload.retryable));
        source.close();
      });
      source.addEventListener("session_aborted", () => {
        reviewAgentStreaming = false;
        transcript.insertAdjacentHTML("beforeend", "<p><strong>Pi analysis aborted.</strong></p>");
        source.close();
      });
    }

    function renderReviewAgentFailure(message, retryable) {
      const transcript = document.querySelector("[data-review-agent-transcript]");
      transcript.innerHTML = "<p role=\\\"alert\\\"><strong>Pi review analysis failed:</strong> " + escapeText(message) + "</p>"
        + (retryable ? "<button type=\\\"button\\\" data-review-agent-retry>Retry safely</button>" : "");
      transcript.querySelector("[data-review-agent-retry]")?.addEventListener("click", () => {
        document.querySelector("[data-review-agent-start]")?.click();
      });
    }

    async function loadRecoveredReviewAgentSessions() {
      const transcript = document.querySelector("[data-review-agent-transcript]");
      if (!transcript) return;
      const response = await fetch("/api/review-agent/" + encodeURIComponent(changeName) + "/sessions");
      if (!response.ok) return;
      const payload = await response.json();
      const completed = (payload.sessions || []).filter((session) => session.status === "completed" && session.responseText);
      if (completed.length === 0) return;
      transcript.innerHTML = "<h3>Recent completed Pi analysis</h3>" + completed.slice(0, 3).map((session) => "<article class=\\\"feedback-record\\\"><p class=\\\"muted\\\">" + escapeText(session.updatedAt || session.createdAt || "") + "</p><pre>" + escapeText(session.responseText || "") + "</pre></article>").join("");
    }

    async function loadFeedback() {
      const response = await fetch("/api/review-feedback/" + encodeURIComponent(changeName));
      if (!response.ok) {
        list.innerHTML = "<p>Unable to load feedback records.</p>";
        return;
      }
      const payload = await response.json();
      loadedFeedback = payload.feedback || [];
      renderDocumentMarkers(loadedFeedback);
      renderCommentsOverview(loadedFeedback);
      wireFeedbackActions(document);
    }

    function renderDocumentMarkers(feedback) {
      document.querySelectorAll("[data-comment-marker], .artifact-marker-list").forEach((item) => item.remove());
      feedback.forEach((record, index) => {
        if (record.anchorState !== "exact" && record.anchorState !== "fuzzy") return;
        const artifact = findArtifact(record.artifactPath);
        if (!artifact) return;
        const marker = document.createElement("button");
        marker.type = "button";
        marker.className = "comment-marker " + record.anchorState;
        marker.dataset.commentMarker = record.id || "";
        marker.textContent = String(index + 1);
        marker.title = "Open " + record.anchorState + " comment";
        const target = findMarkerTarget(artifact, record);
        if (target && target.parentElement) {
          target.parentElement.insertBefore(marker, target);
        } else {
          let markerList = artifact.querySelector(".artifact-marker-list");
          if (!markerList) {
            markerList = document.createElement("div");
            markerList.className = "artifact-marker-list";
            artifact.querySelector(".artifact-content")?.prepend(markerList);
          }
          markerList.append(marker);
        }
      });
    }

    function findMarkerTarget(artifact, record) {
      const selectedText = (record.selectedText || record.anchor?.selectedText || "").trim();
      if (!selectedText) return artifact.querySelector(".artifact-content > .markdown > *");
      const blocks = artifact.querySelectorAll(".artifact-content .markdown > *");
      return Array.from(blocks).find((block) => block.textContent && block.textContent.includes(selectedText)) || blocks[0];
    }

    function renderCommentsOverview(feedback) {
      if (feedback.length === 0) {
        list.innerHTML = "<p>No review companion feedback records yet. Select artifact text to add one.</p>";
        return;
      }
      const statuses = ["open", "addressed", "resolved"];
      list.innerHTML = statuses.map((commentStatus) => {
        const records = feedback.filter((record) => (record.status || "open") === commentStatus);
        return "<section class=\\\"feedback-group\\\"><h3>" + capitalize(commentStatus) + " <span class=\\\"nav-count\\\">" + records.length + "</span></h3>"
          + (records.length === 0 ? "<p class=\\\"muted\\\">No " + commentStatus + " comments.</p>" : records.map(renderFeedbackRecord).join(""))
          + "</section>";
      }).join("");
    }

    function renderFeedbackRecord(record) {
      const id = escapeText(record.id || "");
      const hasMarker = record.anchorState === "exact" || record.anchorState === "fuzzy";
      const body = escapeText(record.body || "");
      const selectedText = escapeText(record.selectedText || record.anchor?.selectedText || "");
      const artifactPath = escapeText(record.artifactPath || "No artifact path");
      const updatedAt = escapeText(record.updatedAt || record.timestamp || "");
      const editCount = Array.isArray(record.editHistory) ? record.editHistory.length : 0;
      const replies = Array.isArray(record.replies) ? record.replies.length : 0;
      return "<article class=\\\"feedback-record\\\" id=\\\"comment-" + id + "\\\">"
        + "<p><strong>" + escapeText(record.severity || "feedback") + "</strong> <span class=\\\"muted\\\">" + updatedAt + (editCount > 0 ? " · edited" : "") + (replies > 0 ? " · " + replies + " replies" : "") + "</span></p>"
        + "<p>" + body + "</p>"
        + "<p><mark>" + selectedText + "</mark></p>"
        + "<p class=\\\"muted\\\">" + artifactPath + " · " + escapeText(record.anchorState || "orphaned") + " · " + escapeText(record.status || "open") + "</p>"
        + "<label><input type=\\\"checkbox\\\" data-feedback-select value=\\\"" + id + "\\\" /> Include in next Ask Pi request</label>"
        + (hasMarker ? "<button type=\\\"button\\\" data-jump-marker=\\\"" + id + "\\\">Jump to marker</button> " : "")
        + "<button type=\\\"button\\\" data-open-thread=\\\"" + id + "\\\">Inspect thread</button> "
        + "<button type=\\\"button\\\" class=\\\"danger-button\\\" data-delete-comment=\\\"" + id + "\\\">Delete comment</button>"
        + "</article>";
    }

    function renderThreadPopover(record, anchorElement) {
      const id = escapeText(record.id || "");
      const replies = Array.isArray(record.replies) ? record.replies : [];
      const editCount = Array.isArray(record.editHistory) ? record.editHistory.length : 0;
      threadPopover.innerHTML = "<div class=\\\"section-heading compact-heading\\\"><div><p class=\\\"eyebrow\\\">Comment thread</p><h2>" + escapeText(record.status || "open") + " feedback</h2></div><button type=\\\"button\\\" class=\\\"secondary-button\\\" data-close-thread>Close</button></div>"
        + "<p>" + escapeText(record.body || "") + "</p>"
        + "<p><mark>" + escapeText(record.selectedText || record.anchor?.selectedText || "") + "</mark></p>"
        + "<p class=\\\"muted\\\">" + escapeText(record.artifactPath || "No artifact path") + " · " + escapeText(record.anchorState || "orphaned") + " · updated " + escapeText(record.updatedAt || record.timestamp || "") + (editCount > 0 ? " · edited" : "") + "</p>"
        + "<div class=\\\"reply-list\\\">" + (replies.length === 0 ? "<p class=\\\"muted\\\">No replies yet.</p>" : replies.map((reply) => "<p><strong>Reply:</strong> " + escapeText(reply.body || "") + " <span class=\\\"muted\\\">" + escapeText(reply.createdAt || "") + "</span></p>").join("")) + "</div>"
        + "<form class=\\\"comment-edit-form\\\" data-comment-id=\\\"" + id + "\\\"><label>Edit comment<textarea name=\\\"body\\\" rows=\\\"4\\\" required>" + escapeText(record.body || "") + "</textarea></label><button type=\\\"submit\\\">Save edit</button></form>"
        + "<form class=\\\"reply-form\\\" data-comment-id=\\\"" + id + "\\\"><input name=\\\"body\\\" placeholder=\\\"Reply to this thread\\\" required /><button type=\\\"submit\\\">Reply</button></form>"
        + "<form class=\\\"status-form\\\" data-comment-id=\\\"" + id + "\\\"><select name=\\\"status\\\"><option value=\\\"open\\\">Open</option><option value=\\\"addressed\\\">Addressed</option><option value=\\\"resolved\\\">Resolved</option></select><button type=\\\"submit\\\">Update status</button></form>"
        + "<button type=\\\"button\\\" class=\\\"danger-button\\\" data-delete-comment=\\\"" + id + "\\\">Delete comment</button>";
      threadPopover.hidden = false;
      positionPopover(threadPopover, anchorElement);
      wireFeedbackActions(threadPopover);
    }

    function wireFeedbackActions(root) {
      root.querySelectorAll("[data-comment-marker]").forEach((button) => button.addEventListener("click", () => {
        const record = loadedFeedback.find((item) => item.id === button.dataset.commentMarker);
        if (record) renderThreadPopover(record, button);
      }));
      root.querySelectorAll("[data-open-thread]").forEach((button) => button.addEventListener("click", () => {
        const record = loadedFeedback.find((item) => item.id === button.dataset.openThread);
        if (record) renderThreadPopover(record, button);
      }));
      root.querySelectorAll("[data-jump-marker]").forEach((button) => button.addEventListener("click", () => {
        const marker = document.querySelector("[data-comment-marker='" + CSS.escape(button.dataset.jumpMarker) + "']");
        marker?.scrollIntoView({ behavior: "smooth", block: "center" });
        marker?.classList.add("emphasized");
        window.setTimeout(() => marker?.classList.remove("emphasized"), 1400);
      }));
      root.querySelectorAll("[data-close-thread]").forEach((button) => button.addEventListener("click", () => {
        threadPopover.hidden = true;
      }));
      root.querySelectorAll("[data-delete-comment]").forEach((button) => {
        if (button.dataset.wired === "true") return;
        button.dataset.wired = "true";
        button.addEventListener("click", async () => {
          const commentId = button.dataset.deleteComment;
          if (!commentId || !window.confirm("Delete this comment and all replies? This cannot be undone.")) return;
          const response = await fetch("/api/review-feedback/" + encodeURIComponent(changeName) + "/" + encodeURIComponent(commentId), {
            method: "DELETE",
          });
          if (!response.ok) {
            const payload = await response.json().catch(() => ({}));
            status.textContent = payload.error || "Unable to delete feedback record.";
            return;
          }
          status.textContent = "Feedback comment deleted.";
          threadPopover.hidden = true;
          await loadFeedback();
        });
      });
      root.querySelectorAll(".reply-form, .status-form, .comment-edit-form").forEach((threadForm) => {
        if (threadForm.dataset.wired === "true") return;
        threadForm.dataset.wired = "true";
        threadForm.addEventListener("submit", async (event) => {
          event.preventDefault();
          const commentId = threadForm.dataset.commentId;
          const kind = threadForm.classList.contains("reply-form") ? "reply" : threadForm.classList.contains("status-form") ? "status" : "edit";
          const response = await fetch("/api/review-feedback/" + encodeURIComponent(changeName) + "/" + encodeURIComponent(commentId) + "/" + kind, {
            method: "POST",
            headers: { "content-type": "application/json" },
            body: JSON.stringify(Object.fromEntries(new FormData(threadForm).entries())),
          });
          if (!response.ok) {
            const payload = await response.json().catch(() => ({}));
            status.textContent = payload.error || "Unable to update feedback record.";
            return;
          }
          status.textContent = kind === "edit" ? "Comment edit saved." : "Feedback thread updated.";
          threadPopover.hidden = true;
          await loadFeedback();
        });
      });
    }

    function selectedArtifactContext() {
      const selection = window.getSelection();
      const selectedText = selection ? selection.toString().trim() : "";
      if (!selection || selectedText === "" || selection.rangeCount === 0) return null;
      const range = selection.getRangeAt(0);
      const node = selection.anchorNode?.nodeType === Node.TEXT_NODE ? selection.anchorNode.parentElement : selection.anchorNode;
      const artifact = node?.closest ? node.closest(".companion-artifact") : null;
      if (!artifact || !artifact.contains(range.commonAncestorContainer)) return null;
      return { artifact, selectedText, range };
    }

    function captureSelection(context) {
      document.querySelectorAll(".companion-artifact.has-selection").forEach((item) => item.classList.remove("has-selection"));
      if (!context) {
        pendingSelection = null;
        affordance.hidden = true;
        return false;
      }
      const { artifact, selectedText, range } = context;
      artifact.classList.add("has-selection");
      const markdown = artifact.querySelector(".artifact-content")?.dataset.artifactMarkdown || "";
      const startOffset = markdown.indexOf(selectedText);
      pendingSelection = {
        artifact,
        selectedText,
        artifactPath: artifact.dataset.artifactPath || "",
        startOffset: startOffset >= 0 ? String(startOffset) : "",
        endOffset: startOffset >= 0 ? String(startOffset + selectedText.length) : "",
        beforeText: startOffset >= 0 ? markdown.slice(Math.max(0, startOffset - 80), startOffset) : "",
        afterText: startOffset >= 0 ? markdown.slice(startOffset + selectedText.length, startOffset + selectedText.length + 80) : "",
        rect: range.getBoundingClientRect(),
      };
      showAffordance(pendingSelection.rect);
      return true;
    }

    function showAffordance(rect) {
      affordance.hidden = false;
      affordance.style.left = Math.min(window.innerWidth - 150, Math.max(12, rect.left + window.scrollX)) + "px";
      affordance.style.top = Math.max(12, rect.top + window.scrollY - 44) + "px";
    }

    function openComposer() {
      if (!pendingSelection) return;
      form.elements.artifactPath.value = pendingSelection.artifactPath;
      form.elements.selectedText.value = pendingSelection.selectedText;
      form.elements.startOffset.value = pendingSelection.startOffset;
      form.elements.endOffset.value = pendingSelection.endOffset;
      form.elements.beforeText.value = pendingSelection.beforeText;
      form.elements.afterText.value = pendingSelection.afterText;
      composer.querySelector("[data-composer-context]").textContent = pendingSelection.artifactPath + " · “" + pendingSelection.selectedText.slice(0, 120) + (pendingSelection.selectedText.length > 120 ? "…" : "") + "”";
      composer.hidden = false;
      affordance.hidden = true;
      if (window.matchMedia("(max-width: 700px)").matches) {
        pendingSelection.artifact.after(composer);
        composer.classList.add("inline-fallback");
      } else {
        document.querySelector(".companion-shell").append(composer);
        composer.classList.remove("inline-fallback");
        positionPopover(composer, pendingSelection.rect);
      }
    }

    function positionPopover(popover, anchor) {
      const rect = anchor && typeof anchor.getBoundingClientRect === "function" ? anchor.getBoundingClientRect() : anchor;
      const left = rect ? Math.min(window.innerWidth - 430, Math.max(12, rect.left + window.scrollX)) : 24;
      const top = rect ? Math.max(12, rect.bottom + window.scrollY + 10) : 80;
      popover.style.left = left + "px";
      popover.style.top = top + "px";
    }

    function findArtifact(artifactPath) {
      if (!artifactPath) return null;
      return document.querySelector(".companion-artifact[data-artifact-path='" + CSS.escape(artifactPath) + "']");
    }

    function escapeText(value) {
      return String(value).replace(/[&<>\"']/g, (character) => {
        if (character === "&") return "&amp;";
        if (character === "<") return "&lt;";
        if (character === ">") return "&gt;";
        if (character === "\\\"") return "&quot;";
        return "&#39;";
      });
    }

    function capitalize(value) {
      return value.charAt(0).toUpperCase() + value.slice(1);
    }

    document.addEventListener("selectionchange", () => {
      captureSelection(selectedArtifactContext());
    });
    document.querySelectorAll(".companion-artifact").forEach((artifact) => {
      artifact.addEventListener("contextmenu", (event) => {
        const context = selectedArtifactContext();
        if (!context || context.artifact !== artifact) return;
        event.preventDefault();
        captureSelection(context);
      });
    });
    affordance.querySelector("[data-open-composer]").addEventListener("click", openComposer);
    composer.querySelector("[data-close-composer]").addEventListener("click", () => {
      composer.hidden = true;
    });
    form.addEventListener("submit", async (event) => {
      event.preventDefault();
      status.textContent = "Saving feedback record…";
      const data = Object.fromEntries(new FormData(form).entries());
      const response = await fetch("/api/review-feedback/" + encodeURIComponent(changeName), {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(data),
      });
      if (!response.ok) {
        const payload = await response.json().catch(() => ({}));
        status.textContent = payload.error || "Unable to save feedback record.";
        return;
      }
      form.reset();
      composer.hidden = true;
      status.textContent = "Saved as a sidecar feedback record.";
      await loadFeedback();
    });
    initializeTabs();
    initializeReviewAgent();
    void loadFeedback();
    void loadRecoveredReviewAgentSessions();
  `;
}
