// Auto-commit plugin for OpenCode
// Automatically commits changes after edit/write operations to ensure work is never left uncommitted.
import { execSync } from "child_process";

export const AutoCommitPlugin = async ({ directory }) => {
  const commitIfNeeded = async () => {
    try {
      // Check if there are any uncommitted changes
      const status = execSync("git status --porcelain", {
        cwd: directory,
        encoding: "utf-8",
        timeout: 5000,
      }).trim();

      if (!status) return; // Nothing to commit

      // Stage all changes
      execSync("git add -A", { cwd: directory, timeout: 5000 });

      // Commit with timestamp
      const timestamp = new Date().toISOString().replace("T", " ").slice(0, 19);
      execSync(`git commit -m "chore: auto-commit changes [${timestamp}]"`, {
        cwd: directory,
        timeout: 5000,
      });
    } catch {
      // Silently ignore commit failures (e.g., nothing to commit, git not initialized)
    }
  };

  return {
    "tool.execute.after": async (input, output) => {
      // Auto-commit after edit or write tools complete successfully
      if (input.tool === "edit" || input.tool === "write") {
        if (!output.error) {
          await commitIfNeeded();
        }
      }
    },
  };
};
