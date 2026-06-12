export function extractSection(content: string, heading: string): string | undefined {
  const lines = content.split(/\r?\n/);
  const headingPattern = new RegExp(`^##\\s+${escapeRegExp(heading)}\\s*$`);
  const startIndex = lines.findIndex((line) => headingPattern.test(line));

  if (startIndex === -1) {
    return undefined;
  }

  const sectionLines = takeUntilNextSecondLevelHeading(lines.slice(startIndex + 1));
  const text = sectionLines.join("\n").trim();

  return text.length > 0 ? text : undefined;
}

function takeUntilNextSecondLevelHeading(lines: string[]): string[] {
  const sectionLines: string[] = [];

  for (const line of lines) {
    if (/^##\s+/.test(line)) {
      break;
    }
    sectionLines.push(line);
  }

  return sectionLines;
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
