import { readFile } from "node:fs/promises";
import { extractSection } from "./markdown.js";
import type {
  DiscoveredSpecFile,
  MalformedScenarioHeading,
  ParsedSpec,
  Requirement,
} from "./types.js";

const requirementHeading = /^### Requirement:\s*(.+?)\s*$/;
const scenarioHeading = /^#### Scenario:\s*(.+?)\s*$/;
const malformedScenarioHeading = /^(#{1,3}|#{5,6}) Scenario:\s*(.+?)\s*$/;

export async function parseSpecFile(specFile: DiscoveredSpecFile): Promise<ParsedSpec> {
  const content = await readFile(specFile.path, "utf8");
  return parseSpecMarkdown(content, specFile);
}

export function parseSpecMarkdown(content: string, specFile: DiscoveredSpecFile): ParsedSpec {
  const headingParse = parseRequirementAndScenarioHeadings(content);

  return {
    name: specFile.name,
    path: specFile.path,
    relativePath: specFile.relativePath,
    purpose: extractSection(content, "Purpose"),
    requirements: headingParse.requirements,
    malformedScenarioHeadings: headingParse.malformedScenarioHeadings,
  };
}

function parseRequirementAndScenarioHeadings(content: string): {
  requirements: Requirement[];
  malformedScenarioHeadings: MalformedScenarioHeading[];
} {
  const requirements: Requirement[] = [];
  const malformedScenarioHeadings: MalformedScenarioHeading[] = [];
  let currentRequirement: Requirement | undefined;

  content.split(/\r?\n/).forEach((line, index) => {
    const lineNumber = index + 1;
    const requirement = matchRequirement(line, lineNumber);
    if (requirement) {
      currentRequirement = requirement;
      requirements.push(requirement);
      return;
    }

    const scenario = matchScenario(line, lineNumber);
    if (scenario) {
      currentRequirement?.scenarios.push(scenario);
      return;
    }

    const malformedScenario = matchMalformedScenario(line, lineNumber);
    if (malformedScenario) {
      malformedScenarioHeadings.push(malformedScenario);
    }
  });

  return { requirements, malformedScenarioHeadings };
}

function matchRequirement(line: string, lineNumber: number): Requirement | undefined {
  const match = line.match(requirementHeading);
  return match ? { title: match[1], scenarios: [], line: lineNumber } : undefined;
}

function matchScenario(line: string, lineNumber: number) {
  const match = line.match(scenarioHeading);
  return match ? { title: match[1], line: lineNumber } : undefined;
}

function matchMalformedScenario(
  line: string,
  lineNumber: number,
): MalformedScenarioHeading | undefined {
  const match = line.match(malformedScenarioHeading);
  return match ? { text: line.trim(), line: lineNumber } : undefined;
}
