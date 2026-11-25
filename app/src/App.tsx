import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
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

function folderName(path: string) {
  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

function noteLines(note: string) {
  return note
    .split(/(?=Graphify )/)
    .map((line) => line.trim())
    .filter(Boolean);
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
  const report = useRef<HTMLDetailsElement>(null);
  const stage = useRef<HTMLElement>(null);
  const box = useRef<HTMLDivElement>(null);
  const [full, setFull] = useState(false);
  const rootsRef = useRef<string[]>([]);
  const busyRef = useRef(false);
  const pendingRoot = useRef<string | null>(null);
  const knownKinds = useRef<string[]>([]);
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
    let unlisten: (() => void) | undefined;
    let cancel = false;
    void listen<string>("root-changed", (event) => {
      if (!rootsRef.current.includes(event.payload)) {
        return;
      }
      if (busyRef.current) {
        pendingRoot.current = event.payload;
        return;
      }
      void refreshRoot(event.payload);
    }).then((stop) => {
      if (cancel) {
        stop();
      } else {
        unlisten = stop;
      }
    });
    return () => {
      cancel = true;
      unlisten?.();
    };
  }, []);

  async function refreshRoot(path: string) {
    busyRef.current = true;
    setBusy(true);
    setError("");
    try {
      const next = await invoke<Graph>("rescan_root", { path });
      setGraph(next);
      const kinds = [...new Set(next.nodes.map((node) => node.kind))];
      setShown((current) => {
        const added = kinds.filter((kind) => !knownKinds.current.includes(kind));
        knownKinds.current = kinds;
        return [...current.filter((kind) => kinds.includes(kind)), ...added];
      });
      setSelected((current) =>
        current && next.nodes.some((node) => node.id === current.id) ? current : null,
      );
      setStatus(`Updated ${path}. Showing ${next.nodes.length} dots.`);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      busyRef.current = false;
      setBusy(false);
      const again = pendingRoot.current;
      pendingRoot.current = null;
      if (again) {
        void refreshRoot(again);
      }
    }
  }

  useEffect(() => {
    const onChange = () => setFull(document.fullscreenElement === stage.current);
    document.addEventListener("fullscreenchange", onChange);
    return () => document.removeEventListener("fullscreenchange", onChange);
  }, []);

  function openReport(message: string) {
    setStatus(message);
    if (report.current) {
      report.current.open = true;
    }
  }

  function toggleFull() {
    if (document.fullscreenElement) {
      void document.exitFullscreen();
      return;
    }
    void stage.current?.requestFullscreen();
  }

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
    busyRef.current = true;
    setError("");
    setStatus("");
    setSelected(null);
    try {
      const next = await invoke<Graph>("scan_roots", { paths });
      setRoots(paths);
      rootsRef.current = paths;
      setGraph(next);
      const kinds = [...new Set(next.nodes.map((node) => node.kind))];
      knownKinds.current = kinds;
      setShown(kinds);
      setStatus("");
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
      busyRef.current = false;
      const again = pendingRoot.current;
      pendingRoot.current = null;
      if (again) {
        void refreshRoot(again);
      }
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
      rootsRef.current = [];
      setRoots([]);
      setGraph(empty);
      setShown([]);
      setSelected(null);
      setStatus("");
      setError("");
      void invoke("clear_watch");
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
    busyRef.current = true;
    setError("");
    try {
      const note = await invoke<string>("keep_canon", {
        root,
        kind: selected.kind,
        name: selected.name,
      });
      await refreshCanon();
      openReport(note);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
      busyRef.current = false;
    }
  }

  async function compareFolder() {
    const root = compareRoot || roots[0];
    if (!root) {
      return;
    }
    setCompareRoot(root);
    setBusy(true);
    busyRef.current = true;
    setError("");
    try {
      const next = await invoke<CanonGap[]>("diff_against_canon", { root });
      setGaps(next);
      setChosen(
        next.filter((gap) => gap.missing).map((gap) => canonKey(gap.kind, gap.name)),
      );
      openReport(
        next.some((gap) => gap.missing)
          ? "Some canon skills or rules are missing from this folder."
          : "This folder already has every canon skill and rule.",
      );
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
      busyRef.current = false;
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
    busyRef.current = true;
    setError("");
    try {
      const note = await invoke<string>("apply_canon_to_folder", {
        root,
        picks,
        confirmed: true,
      });
      if (roots.includes(root)) {
        await load(roots);
      }
      const next = await invoke<CanonGap[]>("diff_against_canon", { root });
      setGaps(next);
      setChosen([]);
      openReport(note);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
      busyRef.current = false;
    }
  }

  function toggleKind(kind: string) {
    setShown((current) =>
      current.includes(kind)
        ? current.filter((item) => item !== kind)
        : [...current, kind],
    );
  }

  const summary =
    roots.length === 0
      ? "No folders open."
      : `Showing ${graph.nodes.length} dots from ${roots.length} ${roots.length === 1 ? "folder" : "folders"}.`;
  const scans = noteLines(graph.note);

  return (
    <>
      <main>
        <details className="report" ref={report}>
          <summary>{summary}</summary>
          <div className="report-body">
            {status ? <p aria-live="polite">{status}</p> : null}
            <p className="note">
              Add more than one folder. Open folders stay watched, and a change
              rescans that folder. A package, skill, API, or database used by two
              folders is one shared dot. An agent file becomes a dot. Drag the
              cloud to turn it. Click a skill or rule, then keep it in the canon.
              Compare a folder and write the missing files only after you confirm.
              This writes a graphify-out folder in each project you add. Password
              files are not opened.
            </p>
            {scans.length > 0 ? (
              <ul className="scans">
                {scans.map((line, index) => (
                  <li key={`${index}-${line}`}>
                    {scans.length === roots.length ? (
                      <span className="kind">{folderName(roots[index])}</span>
                    ) : null}{" "}
                    {line}
                  </li>
                ))}
              </ul>
            ) : null}
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
                <div className="row">
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
                        <option key={path} value={path} title={path}>
                          {folderName(path)}
                        </option>
                      ))}
                    </select>
                  </label>
                  <button type="button" onClick={compareFolder} disabled={busy}>
                    Compare with canon
                  </button>
                </div>
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
                    <span title={path}>{folderName(path)}</span>
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
          </div>
        </details>
        {error ? <p className="alert" role="alert">{error}</p> : null}
        <section className="show" ref={stage} aria-label="Show">
          {busy ? <progress aria-label="Reading the folders" /> : null}
          <div className="bar">
            <h1>NPIN</h1>
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
            <button type="button" onClick={addFolders} disabled={busy}>
              Add folders
            </button>
            <button type="button" onClick={toggleFull}>
              {full ? "Exit full screen" : "Full screen"}
            </button>
          </div>
          <div className="cloud" ref={box}>
            {visible.nodes.length > 0 ? (
              <ForceGraph3D
                graphData={visible}
                width={size.width}
                height={size.height}
                backgroundColor="#07080c"
                nodeAutoColorBy="kind"
                nodeLabel={(node) => `${node.kind}: ${node.name}`}
                nodeRelSize={4}
                onNodeClick={(node) => setSelected(node as Node)}
              />
            ) : (
              <p className="empty">Add a folder to draw the cloud.</p>
            )}
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
                    <p>Uses the folder chosen in the menu above.</p>
                  </>
                ) : null}
              </>
            ) : (
              <p>Click a dot in the cloud.</p>
            )}
          </aside>
        </section>
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
