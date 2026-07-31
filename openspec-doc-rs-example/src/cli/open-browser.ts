import { spawn } from "node:child_process";

export interface OpenBrowserResult {
  opened: boolean;
  error?: string;
}

export async function openBrowser(url: string): Promise<OpenBrowserResult> {
  const command = browserOpenCommand(url);
  if (!command) {
    return {
      opened: false,
      error: `Opening a browser is not supported on ${process.platform}.`,
    };
  }

  try {
    const child = spawn(command.command, command.args, {
      detached: true,
      stdio: "ignore",
    });
    child.unref();
    return { opened: true };
  } catch (error) {
    return {
      opened: false,
      error: error instanceof Error ? error.message : "Unknown browser launch failure.",
    };
  }
}

function browserOpenCommand(url: string): { command: string; args: string[] } | null {
  if (process.platform === "darwin") {
    return { command: "open", args: [url] };
  }

  if (process.platform === "win32") {
    return { command: "cmd", args: ["/c", "start", "", url] };
  }

  if (process.platform === "linux") {
    return { command: "xdg-open", args: [url] };
  }

  return null;
}
