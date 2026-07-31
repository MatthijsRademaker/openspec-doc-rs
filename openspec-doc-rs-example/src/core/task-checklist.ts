export interface TaskChecklistItem {
  line: number;
  checked: boolean;
  text: string;
}

export const TASK_CHECKLIST_PATTERN = /^(\s*)- \[( |x|X)\](\s+.*)$/;

export function parseTaskChecklist(markdown: string): TaskChecklistItem[] {
  return markdown.split(/\r?\n/).flatMap((lineText, index) => {
    const match = lineText.match(TASK_CHECKLIST_PATTERN);
    if (!match) return [];
    return [
      {
        line: index + 1,
        checked: match[2].toLowerCase() === "x",
        text: match[3].trim(),
      },
    ];
  });
}
