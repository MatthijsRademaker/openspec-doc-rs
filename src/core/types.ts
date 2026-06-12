export type WarningCode =
  | "missing-purpose"
  | "requirement-missing-scenario"
  | "malformed-scenario-heading"
  | "missing-change-artifact";

export interface FatalError {
  code: "missing-config" | "scan-failed";
  message: string;
  path?: string;
}

export interface ProjectWarning {
  code: WarningCode;
  message: string;
  path: string;
}

export interface DiscoveredSpecFile {
  path: string;
  relativePath: string;
  name: string;
}

export interface DiscoveredChangeFile {
  path: string;
  relativePath: string;
}

export interface DiscoveredChange {
  name: string;
  path: string;
  relativePath: string;
  files: DiscoveredChangeFile[];
}

export interface ScannedProject {
  rootPath: string;
  openspecPath: string;
  configPath: string;
  specFiles: DiscoveredSpecFile[];
  activeChanges: DiscoveredChange[];
  archivedChanges: DiscoveredChange[];
}

export interface Requirement {
  title: string;
  scenarios: Scenario[];
  line: number;
}

export interface Scenario {
  title: string;
  line: number;
}

export interface MalformedScenarioHeading {
  text: string;
  line: number;
}

export interface ParsedSpec {
  name: string;
  path: string;
  relativePath: string;
  purpose?: string;
  requirements: Requirement[];
  malformedScenarioHeadings: MalformedScenarioHeading[];
}

export interface TaskCounts {
  total: number;
  complete: number;
  incomplete: number;
}

export interface ParsedChange {
  name: string;
  path: string;
  relativePath: string;
  hasProposal: boolean;
  hasDesign: boolean;
  hasTasks: boolean;
  hasSpecDeltas: boolean;
  taskCounts: TaskCounts;
  archived: boolean;
}

export interface ParsedProject {
  scanned: ScannedProject;
  specs: ParsedSpec[];
  activeChanges: ParsedChange[];
  archivedChanges: ParsedChange[];
}

export interface ProjectSummary {
  rootPath: string;
  configPath: string;
  currentSpecs: number;
  activeChanges: number;
  archivedChanges: number;
  requirements: number;
  scenarios: number;
  tasks: TaskCounts;
  warnings: number;
}

export interface AnalysisResult {
  summary: ProjectSummary;
  warnings: ProjectWarning[];
}

export type ScanResult =
  | { ok: true; project: ScannedProject }
  | { ok: false; fatalError: FatalError };
