import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import "./App.css";

type Found = {
  kind: string;
  name: string;
  detail: string;
};

function App() {
  const [rows, setRows] = useState<Found[]>([]);
  const [status, setStatus] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  async function chooseFolder() {
    const selected = await open({ directory: true, multiple: false });
    if (selected === null || Array.isArray(selected)) {
      return;
    }
    setBusy(true);
    setError("");
    setStatus("");
    try {
      const found = await invoke<Found[]>("scan_folder", { path: selected });
      setRows(found);
      setStatus(`Found ${found.length} things.`);
    } catch (reason) {
      setRows([]);
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <header>
        <h1>One folder</h1>
        <button type="button" onClick={chooseFolder} disabled={busy}>
          Open folder
        </button>
      </header>
      <main>
        {busy ? <progress aria-label="Reading the folder" /> : null}
        {error ? (
          <p role="alert">{error}</p>
        ) : (
          <p aria-live="polite">{status}</p>
        )}
        <p className="note">
          This reads project files in the folder you pick and writes a
          graphify-out folder there. Password files are not opened.
        </p>
        {rows.length > 0 ? (
          <ul>
            {rows.map((row) => (
              <li key={`${row.kind}:${row.name}`}>
                <span className="kind">{row.kind}</span>
                <span className="name">{row.name}</span>
                <span className="detail">{row.detail}</span>
              </li>
            ))}
          </ul>
        ) : null}
      </main>
    </>
  );
}

export default App;
