import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import App from "./App";
import type { Api, System, Workspace } from "./api";

function fakeApi(): Api {
  const workspaces: Workspace[] = [];
  const systems: Record<string, System[]> = {};
  return {
    listWorkspaces: vi.fn(async () => [...workspaces]),
    createWorkspace: vi.fn(async (name: string) => {
      const ws = { id: `ws-${workspaces.length}`, name, created_at: "2026-01-01T00:00:00Z" };
      workspaces.push(ws);
      systems[ws.id] = [];
      return ws;
    }),
    listSystems: vi.fn(async (id: string) => [...(systems[id] ?? [])]),
    createSystem: vi.fn(async (id: string, name: string) => {
      const s = { id: `s-${name}`, name, description: null, created_at: "2026-01-01T00:00:00Z" };
      systems[id]?.push(s);
      return s;
    }),
  };
}

describe("App", () => {
  it("creates a workspace and a system", async () => {
    render(<App api={fakeApi()} />);

    fireEvent.change(screen.getByLabelText("New workspace name"), { target: { value: "Acme" } });
    fireEvent.click(screen.getByRole("button", { name: "Add" }));
    expect(await screen.findByRole("heading", { name: "Acme" })).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("New system name"), { target: { value: "Portal" } });
    fireEvent.click(screen.getByRole("button", { name: "Add system" }));
    await waitFor(() => expect(screen.getByLabelText("Systems")).toHaveTextContent("Portal"));
  });

  it("shows command errors without internals", async () => {
    const api = fakeApi();
    api.createWorkspace = vi.fn(async () => {
      throw { code: "invalid_input", message: "Name must not be empty." };
    });
    render(<App api={api} />);
    fireEvent.change(screen.getByLabelText("New workspace name"), { target: { value: "x" } });
    fireEvent.click(screen.getByRole("button", { name: "Add" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Name must not be empty.");
  });
});
