import { useCallback, useEffect, useState, type FormEvent } from "react";
import { isCommandError, type Api, type System, type Workspace } from "./api";

function errorText(e: unknown): string {
  return isCommandError(e) ? e.message : "Something went wrong.";
}

export default function App({ api }: { api: Api }) {
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [selected, setSelected] = useState<Workspace | null>(null);
  const [systems, setSystems] = useState<System[]>([]);
  const [wsName, setWsName] = useState("");
  const [sysName, setSysName] = useState("");
  const [error, setError] = useState<string | null>(null);

  const loadWorkspaces = useCallback(async () => {
    try {
      setWorkspaces(await api.listWorkspaces());
    } catch (e) {
      setError(errorText(e));
    }
  }, [api]);

  useEffect(() => {
    api.listWorkspaces().then(setWorkspaces, (e: unknown) => setError(errorText(e)));
  }, [api]);

  useEffect(() => {
    if (!selected) return;
    api.listSystems(selected.id).then(setSystems, (e: unknown) => setError(errorText(e)));
  }, [api, selected]);

  async function onCreateWorkspace(e: FormEvent) {
    e.preventDefault();
    setError(null);
    try {
      const ws = await api.createWorkspace(wsName);
      setWsName("");
      await loadWorkspaces();
      setSelected(ws);
    } catch (err) {
      setError(errorText(err));
    }
  }

  async function onCreateSystem(e: FormEvent) {
    e.preventDefault();
    if (!selected) return;
    setError(null);
    try {
      await api.createSystem(selected.id, sysName, null);
      setSysName("");
      setSystems(await api.listSystems(selected.id));
    } catch (err) {
      setError(errorText(err));
    }
  }

  return (
    <div className="app">
      <aside className="sidebar">
        <h1>Quality Report Tool</h1>
        <h2>Workspaces</h2>
        <ul className="list" aria-label="Workspaces">
          {workspaces.map((ws) => (
            <li key={ws.id}>
              <button
                className={ws.id === selected?.id ? "item selected" : "item"}
                onClick={() => setSelected(ws)}
              >
                {ws.name}
              </button>
            </li>
          ))}
        </ul>
        <form onSubmit={onCreateWorkspace} className="inline-form">
          <input
            aria-label="New workspace name"
            placeholder="New workspace"
            value={wsName}
            maxLength={200}
            onChange={(e) => setWsName(e.target.value)}
          />
          <button type="submit" disabled={!wsName.trim()}>
            Add
          </button>
        </form>
      </aside>

      <main className="content">
        {error && (
          <p role="alert" className="error">
            {error}
          </p>
        )}
        {selected ? (
          <>
            <h2>{selected.name}</h2>
            <h3>Systems</h3>
            {systems.length === 0 ? (
              <p className="muted">No systems yet.</p>
            ) : (
              <ul className="list" aria-label="Systems">
                {systems.map((s) => (
                  <li key={s.id} className="card">
                    {s.name}
                  </li>
                ))}
              </ul>
            )}
            <form onSubmit={onCreateSystem} className="inline-form">
              <input
                aria-label="New system name"
                placeholder="New system"
                value={sysName}
                maxLength={200}
                onChange={(e) => setSysName(e.target.value)}
              />
              <button type="submit" disabled={!sysName.trim()}>
                Add system
              </button>
            </form>
          </>
        ) : (
          <p className="muted">Create or select a workspace to begin.</p>
        )}
      </main>
    </div>
  );
}
