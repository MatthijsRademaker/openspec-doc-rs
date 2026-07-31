import type { ProjectWarning, TaskCounts } from "./types.js";

export type LifecyclePhaseId = "explore" | "propose" | "review" | "apply" | "archive";

export interface LifecyclePhase {
  id: LifecyclePhaseId;
  label: string;
  description: string;
}

export interface LifecycleAction {
  label: string;
  summary: string;
  href?: string;
  command?: string;
}

export interface LifecycleGuidance {
  phase: LifecyclePhase;
  summary: string;
  reasons: string[];
  actions: LifecycleAction[];
  safetyNotes: string[];
}

export interface ChangeLifecycleInput {
  name: string;
  archived: boolean;
  hasProposal: boolean;
  hasDesign: boolean;
  hasTasks: boolean;
  hasSpecDeltas: boolean;
  taskCounts: TaskCounts;
  warnings: ProjectWarning[];
}

export interface ProjectLifecycleGuidance {
  phase: LifecyclePhase;
  summary: string;
  phases: readonly LifecyclePhase[];
  activeQueue: Array<{ name: string; guidance: LifecycleGuidance }>;
  actions: LifecycleAction[];
  safetyNotes: string[];
}

export const LIFECYCLE_PHASES: readonly LifecyclePhase[] = [
  {
    id: "explore",
    label: "Explore",
    description:
      "Clarify the idea, problem, constraints, and expected outcome before creating a change.",
  },
  {
    id: "propose",
    label: "Propose",
    description: "Create or refine OpenSpec proposal, design, tasks, and spec delta artifacts.",
  },
  {
    id: "review",
    label: "Review",
    description:
      "Inspect proposal artifacts, warnings, and feedback before implementation or completion.",
  },
  {
    id: "apply",
    label: "Apply",
    description: "Implement incomplete tasks, apply approved updates, and run validation.",
  },
  {
    id: "archive",
    label: "Archive",
    description: "Validate completed work and explicitly archive the OpenSpec change.",
  },
] as const;

const PHASE_BY_ID = new Map(LIFECYCLE_PHASES.map((phase) => [phase.id, phase]));

export function getLifecyclePhase(id: LifecyclePhaseId): LifecyclePhase {
  const phase = PHASE_BY_ID.get(id);
  if (!phase) {
    throw new Error(`Unknown lifecycle phase: ${id}`);
  }
  return phase;
}

export function buildProjectLifecycleGuidance(
  activeChanges: ChangeLifecycleInput[],
): ProjectLifecycleGuidance {
  const safetyNotes = [
    "Local lifecycle buttons can run allowlisted commands or update task checkboxes only after explicit confirmation.",
    "Handoff actions provide Pi or CLI guidance without mutating proposal artifacts or code.",
  ];

  if (activeChanges.length === 0) {
    return {
      phase: getLifecyclePhase("explore"),
      summary:
        "No active OpenSpec changes were found. Explore and clarify the next idea before proposing a change.",
      phases: LIFECYCLE_PHASES,
      activeQueue: [],
      actions: [
        {
          label: "Explore an idea with Pi",
          summary:
            "Ask Pi or your agent workflow to help clarify the idea before creating a proposal.",
          command: "Ask Pi: explore this OpenSpec change idea",
        },
        {
          label: "List OpenSpec changes",
          summary: "Confirm the project has no active changes.",
          command: "openspec list",
        },
      ],
      safetyNotes,
    };
  }

  const activeQueue = activeChanges.map((change) => ({
    name: change.name,
    guidance: inferChangeLifecycleGuidance(change),
  }));

  return {
    phase: activeQueue[0]?.guidance.phase ?? getLifecyclePhase("review"),
    summary: `${activeChanges.length} active OpenSpec change(s) are in the lifecycle queue. Continue existing work before starting unrelated changes.`,
    phases: LIFECYCLE_PHASES,
    activeQueue,
    actions: [
      {
        label: "Review active queue",
        summary: "Open the active changes index to triage lifecycle phases.",
        href: "/changes",
      },
      {
        label: "Validate active work",
        summary: "Run validation before applying or archiving changes.",
        command: "openspec validate <change-name>",
      },
    ],
    safetyNotes,
  };
}

export function inferChangeLifecycleGuidance(change: ChangeLifecycleInput): LifecycleGuidance {
  if (change.archived) {
    return {
      phase: getLifecyclePhase("archive"),
      summary: "This change is archived; the active lifecycle is complete.",
      reasons: ["The change is located in the OpenSpec archive."],
      actions: [],
      safetyNotes: [
        "Archived change pages do not suggest active Propose, Review, Apply, or Archive actions.",
      ],
    };
  }

  const missingArtifacts = missingRequiredArtifacts(change);
  if (missingArtifacts.length > 0) {
    return activeGuidance("propose", change.name, {
      summary: "This active change needs Propose-phase attention before review or implementation.",
      reasons: missingArtifacts.map((artifact) => `Missing ${artifact}.`),
    });
  }

  if (change.taskCounts.total > 0 && change.taskCounts.incomplete === 0) {
    if (change.warnings.length === 0) {
      return activeGuidance("archive", change.name, {
        summary: "Tasks are complete and no change warnings were found; validate before archiving.",
        reasons: [
          `All ${change.taskCounts.total} task(s) are complete.`,
          "Required proposal artifacts are present.",
          "No analyzer warnings apply to this active change.",
        ],
      });
    }

    return activeGuidance("review", change.name, {
      summary:
        "Tasks are complete, but warnings still need review before archive guidance is safe.",
      reasons: [
        `All ${change.taskCounts.total} task(s) are complete.`,
        `${change.warnings.length} analyzer warning(s) still apply to this change.`,
      ],
    });
  }

  if (change.taskCounts.incomplete > 0 && change.taskCounts.complete > 0) {
    return activeGuidance("apply", change.name, {
      summary:
        "Implementation is in progress; continue applying incomplete tasks and validate the result.",
      reasons: [
        `${change.taskCounts.complete}/${change.taskCounts.total} task(s) are complete.`,
        `${change.taskCounts.incomplete} task(s) remain incomplete.`,
      ],
    });
  }

  return activeGuidance("review", change.name, {
    summary:
      "Proposal artifacts are present and implementation has not clearly started; review before applying.",
    reasons: [
      "Proposal, design, tasks, and spec delta artifacts are present.",
      change.taskCounts.total === 0
        ? "No task checklist items were found, so review the tasks artifact before implementation."
        : "Task progress does not show implementation work in progress yet.",
      ...(change.warnings.length > 0
        ? [`${change.warnings.length} analyzer warning(s) may need review.`]
        : []),
    ],
  });
}

function missingRequiredArtifacts(change: ChangeLifecycleInput): string[] {
  return [
    { present: change.hasProposal, label: "proposal.md" },
    { present: change.hasDesign, label: "design.md" },
    { present: change.hasTasks, label: "tasks.md" },
    { present: change.hasSpecDeltas, label: "spec delta artifact" },
  ]
    .filter((artifact) => !artifact.present)
    .map((artifact) => artifact.label);
}

function activeGuidance(
  phaseId: LifecyclePhaseId,
  changeName: string,
  details: Pick<LifecycleGuidance, "summary" | "reasons">,
): LifecycleGuidance {
  return {
    phase: getLifecyclePhase(phaseId),
    summary: details.summary,
    reasons: details.reasons,
    actions: actionsForPhase(phaseId, changeName),
    safetyNotes: safetyNotesForPhase(phaseId),
  };
}

function actionsForPhase(phaseId: LifecyclePhaseId, changeName: string): LifecycleAction[] {
  switch (phaseId) {
    case "explore":
      return [];
    case "propose":
      return [
        {
          label: "Refine proposal artifacts",
          summary:
            "Use Pi or your agent workflow to create or update proposal, design, tasks, and spec deltas.",
          command: `Ask Pi: propose or continue ${changeName}`,
        },
        {
          label: "Validate proposal state",
          summary: "Check OpenSpec artifacts after refinement.",
          command: `openspec validate ${changeName}`,
        },
      ];
    case "review":
      return [
        {
          label: "Open review page",
          summary: "Review proposal, design, tasks, spec deltas, and analyzer warnings.",
          href: `/changes/${encodeURIComponent(changeName)}/review`,
        },
        {
          label: "Open review companion",
          summary:
            "Capture advisory sidecar comments or request Pi analysis before implementation.",
          href: `/changes/${encodeURIComponent(changeName)}/companion`,
        },
        {
          label: "Validate before apply",
          summary: "Run validation before marking implementation ready.",
          command: `openspec validate ${changeName}`,
        },
      ];
    case "apply":
      return [
        {
          label: "Apply implementation tasks",
          summary:
            "Use Pi or the agent workflow to implement remaining tasks with explicit approval.",
          command: `/opsx-apply ${changeName}`,
        },
        {
          label: "Validate implementation",
          summary: "Run OpenSpec validation before considering the change complete.",
          command: `openspec validate ${changeName}`,
        },
      ];
    case "archive":
      return [
        {
          label: "Validate before archive",
          summary: "Confirm the change is valid before moving it into the archive.",
          command: `openspec validate ${changeName}`,
        },
        {
          label: "Archive explicitly",
          summary: "Archive from the CLI only after validation passes.",
          command: `openspec archive ${changeName}`,
        },
      ];
  }
}

function safetyNotesForPhase(phaseId: LifecyclePhaseId): string[] {
  const notes = [
    "Local lifecycle buttons can run allowlisted commands or update task checkboxes only after explicit confirmation.",
    "Handoff actions do not directly edit proposal, design, spec delta, or code files.",
  ];

  if (phaseId === "apply") {
    notes.push("Artifact and code updates require explicit user approval before they are applied.");
    notes.push("Run validation before marking implementation complete.");
  }

  if (phaseId === "archive") {
    notes.push("Run validation before archive guidance is acted on.");
    notes.push(
      "Archive buttons run the explicit local OpenSpec archive command after confirmation.",
    );
  }

  return notes;
}
