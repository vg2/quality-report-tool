// Tauri isolation hook: every IPC message from the UI passes through this
// sandboxed iframe before reaching Rust. We reject any command that is not
// on the app's allowlist, as defence in depth alongside capabilities.
// Plugin/internal IPC (`plugin:...`) is left to Tauri's own permission checks.
const ALLOWED_APP_COMMANDS = new Set([
  "list_workspaces",
  "create_workspace",
  "list_systems",
  "create_system",
]);

window.__TAURI_ISOLATION_HOOK__ = (payload) => {
  const cmd = payload && payload.cmd;
  if (typeof cmd !== "string") return null;
  if (cmd.startsWith("plugin:") || ALLOWED_APP_COMMANDS.has(cmd)) return payload;
  console.warn("Blocked IPC command:", cmd);
  return null;
};
