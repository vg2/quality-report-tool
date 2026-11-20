// Typed client for the Rust IPC commands. Every command the UI may call is
// listed here and must also be granted in src-tauri/capabilities/main.json.
import { invoke } from "@tauri-apps/api/core";

export interface Workspace {
  id: string;
  name: string;
  created_at: string;
}

export interface System {
  id: string;
  name: string;
  description: string | null;
  created_at: string;
}

/** Error shape returned by Rust commands (no internal paths or secrets). */
export interface CommandError {
  code: "invalid_input" | "not_found" | "storage" | "internal";
  message: string;
}

export function isCommandError(e: unknown): e is CommandError {
  return typeof e === "object" && e !== null && "code" in e && "message" in e;
}

export const api = {
  listWorkspaces: () => invoke<Workspace[]>("list_workspaces"),
  createWorkspace: (name: string) => invoke<Workspace>("create_workspace", { name }),
  listSystems: (workspaceId: string) => invoke<System[]>("list_systems", { workspaceId }),
  createSystem: (workspaceId: string, name: string, description: string | null) =>
    invoke<System>("create_system", { workspaceId, name, description }),
};

export type Api = typeof api;
