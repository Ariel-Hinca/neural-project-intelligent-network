import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import ForceGraph3D from "react-force-graph-3d";
import "./App.css";

type Node = {
  id: string;
  kind: string;
  name: string;
  detail: string;
  file: string;
};

type Link = {
  source: string;
  target: string;
};

type Graph = {
  nodes: Node[];
  links: Link[];
  note: string;
};

const empty: Graph = { nodes: [], links: [], note: "" };

type CanonEntry = { kind: string; name: string };
type CanonGap = { kind: string; name: string; missing: boolean };

function canonKey(kind: string, name: string) {
  return `${kind}\n${name}`;
}

function App() {
  const [roots, setRoots] = useState<string[]>([]);
  const [graph, setGraph] = useState<Graph>(empty);
  const [shown, setShown] = useState<string[]>([]);
  const [selected, setSelected] = useState<Node | null>(null);
  const [status, setStatus] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [canon, setCanon] = useState<CanonEntry[]>([]);
  const [gaps, setGaps] = useState<CanonGap[] | null>(null);
  const [compareRoot, setCompareRoot] = useState("");
  const [chosen, setChosen] = useState<string[]>([]);
  const confirmDialog = useRef<HTMLDialogElement>(null);
  const box = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState({ width: 640, height: 480 });

  useEffect(() => {
    void invoke<CanonEntry[]>("canon_list")
      .then(setCanon)
      .catch((reason: unknown) =>
        setError(reason instanceof Error ? reason.message : String(reason)),
      );
  }, []);

  const kinds = useMemo(
    () => [...new Set(graph.nodes.map((node) => node.kind))],
    [graph],
  );

  const visible = useMemo(() => {
    const allow = new Set(shown);
    const nodes = graph.nodes.filter((node) => allow.has(node.kind));
    const ids = new Set(nodes.map((node) => node.id));
    const links = graph.links.filter(
      (link) => ids.has(link.source) && ids.has(link.target),
    );
    return {
      nodes: nodes.map((node) => ({ ...node })),
      links: links.map((link) => ({ ...link })),
    };
  }, [graph, shown]);

  useEffect(() => {
    const el = box.current;
    if (!el) {
      return;
    }
    const update = () =>
      setSize({ width: el.clientWidth, height: el.clientHeight });
    update();
    const observer = new ResizeObserver(update);
    observer.observe(el);
    return () => observer.disconnect();
  }, [graph.nodes.length]);

  async function load(paths: string[]) {
    setBusy(true);
    setError("");
    setStatus("");
    setSelected(null);
    try {
      const next = await invoke<Graph>("scan_roots", { paths });
      setRoots(paths);
      setGraph(next);
      setShown([...new Set(next.nodes.map((node) => node.kind))]);
      setStatus(
        `Showing ${next.nodes.length} dots from ${paths.length} ${paths.length === 1 ? "folder" : "folders"}.`,
      );
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  }

  async function addFolders() {
    const picked = await open({ directory: true, multiple: true });
    if (picked === null) {
      return;
    }
    const paths = Array.isArray(picked) ? picked : [picked];
    const next = [...roots];
    for (const path of paths) {
      if (!next.includes(path)) {
        next.push(path);
      }
    }
    await load(next);
  }

  function removeRoot(path: string) {
    const next = roots.filter((item) => item !== path);
    if (next.length === 0) {
      setRoots([]);
      setGraph(empty);
      setShown([]);
      setSelected(null);
      setStatus("");
      setError("");
      return;
    }
    void load(next);
  }

  async function refreshCanon() {
    const next = await invoke<CanonEntry[]>("canon_list");
    setCanon(next);
  }

  async function keepSelected() {
    if (!selected || roots.length === 0) {
      return;
    }
    const root = compareRoot || roots[0];
    setBusy(true);
    setError("");
    try {
      const note = await invoke<string>("keep_canon", {
        root,
        kind: selected.kind,
        name: selected.name,
      });
      await refreshCanon();
      setStatus(note);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  }

  async function compareFolder() {
    const root = compareRoot || roots[0];
    if (!root) {
      return;
    }
    setCompareRoot(root);
    setBusy(true);
    setError("");
    try {
      const next = await invoke<CanonGap[]>("diff_against_canon", { root });
      setGaps(next);
      setChosen(
        next.filter((gap) => gap.missing).map((gap) => canonKey(gap.kind, gap.name)),
      );
      setStatus(
        next.some((gap) => gap.missing)
          ? "Some canon skills or rules are missing from this folder."
          : "This folder already has every canon skill and rule.",
      );
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  }

  function askApply() {
    if (chosen.length === 0) {
      setError("Choose a missing skill or rule.");
      return;
    }
    const dialog = confirmDialog.current;
    if (!dialog) {
      return;
    }
    dialog.returnValue = "";
    dialog.showModal();
  }

  async function onConfirmClose() {
    const dialog = confirmDialog.current;
    if (!dialog) {
      return;
    }
    const choice = dialog.returnValue;
    dialog.returnValue = "";
    if (choice !== "confirm") {
      return;
    }
    const root = compareRoot || roots[0];
    const picks = chosen.map((key) => {
      const [kind, name] = key.split("\n");
      return { kind, name };
    });
    setBusy(true);
    setError("");
    try {
      const note = await invoke<string>("apply_canon_to_folder", {
        root,
        picks,
        confirmed: true,
      });
      setStatus(note);
      if (roots.includes(root)) {
        await load(roots);
      }
      const next = await invoke<CanonGap[]>("diff_against_canon", { root });
      setGaps(next);
      setChosen([]);
      setStatus(note);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  }

  function toggleKind(kind: string) {
    setShown((current) =>
      current.includes(kind)
        ? current.filter((item) => item !== kind)
        : [...current, kind],
    );
  }

  return (
    <>
      <header>
        <h1>God's eye</h1>
        <button type="button" onClick={addFolders} disabled={busy}>
          Add folders
        </button>
      </header>
      <main>
        {busy ? <progress aria-label="Reading the folders" /> : null}
        {error ? <p role="alert">{error}</p> : <p aria-live="polite">{status}</p>}
        <p className="note">
          Add more than one folder. A package, skill, API, or database used by
          two folders is one shared dot. Drag the cloud to turn it. Click a
          skill or rule, then keep it in the canon. Compare a folder and write
          the missing files only after you confirm. This writes a graphify-out
          folder in each project you add. Password files are not opened.{" "}
          {graph.note}
        </p>
        <fieldset>
          <legend>Canon</legend>
          {canon.length === 0 ? (
            <p>No skills or rules kept yet.</p>
          ) : (
            <ul className="canon">
              {canon.map((entry) => (
                <li key={canonKey(entry.kind, entry.name)}>
                  <span className="kind">{entry.kind}</span> {entry.name}
                </li>
              ))}
            </ul>
          )}
          {roots.length > 0 ? (
            <>
              <label>
                Folder
                <select
                  value={compareRoot || roots[0]}
                  onChange={(event) => {
                    setCompareRoot(event.target.value);
                    setGaps(null);
                    setChosen([]);
                  }}
                >
                  {roots.map((path) => (
                    <option key={path} value={path}>
                      {path}
                    </option>
                  ))}
                </select>
              </label>
              <button type="button" onClick={compareFolder} disabled={busy}>
                Compare with canon
              </button>
            </>
          ) : null}
          {gaps ? (
            <ul className="canon">
              {gaps.map((gap) => (
                <li key={canonKey(gap.kind, gap.name)}>
                  {gap.missing ? (
                    <label>
                      <input
                        type="checkbox"
                        checked={chosen.includes(canonKey(gap.kind, gap.name))}
                        onChange={() => {
                          const key = canonKey(gap.kind, gap.name);
                          setChosen((current) =>
                            current.includes(key)
                              ? current.filter((item) => item !== key)
                              : [...current, key],
                          );
                        }}
                      />
                      Missing {gap.kind} {gap.name}
                    </label>
                  ) : (
                    <span>
                      {gap.kind} {gap.name} is already in this folder.
                    </span>
                  )}
                </li>
              ))}
            </ul>
          ) : null}
          {gaps?.some((gap) => gap.missing) ? (
            <button type="button" onClick={askApply} disabled={busy}>
              Write into this folder
            </button>
          ) : null}
        </fieldset>
        {roots.length > 0 ? (
          <ul className="roots">
            {roots.map((path) => (
              <li key={path}>
                <span>{path}</span>
                <button
                  type="button"
                  onClick={() => removeRoot(path)}
                  disabled={busy}
                >
                  Remove
                </button>
              </li>
            ))}
          </ul>
        ) : null}
        {kinds.length > 0 ? (
          <fieldset>
            <legend>Show</legend>
            {kinds.map((kind) => (
              <label key={kind}>
                <input
                  type="checkbox"
                  checked={shown.includes(kind)}
                  onChange={() => toggleKind(kind)}
                />
                {kind}
              </label>
            ))}
          </fieldset>
        ) : null}
        <div className="stage">
          <div className="cloud" ref={box}>
            {visible.nodes.length > 0 ? (
              <ForceGraph3D
                graphData={visible}
                width={size.width}
                height={size.height}
                backgroundColor="#0b0d12"
                nodeAutoColorBy="kind"
                nodeLabel={(node) => `${node.kind}: ${node.name}`}
                nodeRelSize={4}
                onNodeClick={(node) => setSelected(node as Node)}
              />
            ) : null}
          </div>
          <aside>
            <h2>Selected dot</h2>
            {selected ? (
              <>
                <p>
                  <span className="kind">{selected.kind}</span> {selected.name}
                </p>
                <p>{selected.file || "This dot has no file."}</p>
                <p>{selected.detail}</p>
                {selected.kind === "Skill" || selected.kind === "Rule" ? (
                  <>
                    <button type="button" onClick={keepSelected} disabled={busy || roots.length === 0}>
                      Keep in canon
                    </button>
                    <p>Uses the folder chosen under Canon.</p>
                  </>
                ) : null}
              </>
            ) : (
              <p>Click a dot in the cloud.</p>
            )}
          </aside>
        </div>
      </main>
      <dialog
        ref={confirmDialog}
        aria-labelledby="confirm-title"
        onClose={() => void onConfirmClose()}
      >
        <h1 id="confirm-title">Write these into the folder?</h1>
        <p>{compareRoot || roots[0]}</p>
        <ul>
          {chosen.map((key) => {
            const [kind, name] = key.split("\n");
            return (
              <li key={key}>
                {kind} {name}
              </li>
            );
          })}
        </ul>
        <form method="dialog">
          <button value="cancel" autoFocus>
            Cancel
          </button>
          <button value="confirm">Write files</button>
        </form>
      </dialog>
    </>
  );
}

export default App;
