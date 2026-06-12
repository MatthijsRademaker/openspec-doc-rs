import type { TaskCounts } from "./types.js";

export function countTasks(content: string): TaskCounts {
  const total = [...content.matchAll(/^- \[[ xX]\]\s+/gm)].length;
  const complete = [...content.matchAll(/^- \[[xX]\]\s+/gm)].length;

  return {
    total,
    complete,
    incomplete: total - complete,
  };
}

export function emptyTaskCounts(): TaskCounts {
  return { total: 0, complete: 0, incomplete: 0 };
}
