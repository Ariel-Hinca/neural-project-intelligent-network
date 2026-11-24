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

function App() {
  const [graph, setGraph] = useState<Graph>(empty);
  const [shown, setShown] = useState<string[]>([]);
  const [selected, setSelected] = useState<Node | null>(null);
  const [status, setStatus] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const box = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState({ width: 640, height: 480 });

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

  async function chooseFolder() {
    const selectedPath = await open({ directory: true, multiple: false });
    if (selectedPath === null || Array.isArray(selectedPath)) {
      return;
    }
    setBusy(true);
    setError("");
    setStatus("");
    setSelected(null);
    try {
      const next = await invoke<Graph>("scan_folder", { path: selectedPath });
      setGraph(next);
      setShown([...new Set(next.nodes.map((node) => node.kind))]);
      setStatus(`Showing ${next.nodes.length} dots.`);
    } catch (reason) {
      setGraph(empty);
      setShown([]);
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
        <h1>One folder</h1>
        <button type="button" onClick={chooseFolder} disabled={busy}>
          Open folder
        </button>
      </header>
      <main>
        {busy ? <progress aria-label="Reading the folder" /> : null}
        {error ? <p role="alert">{error}</p> : <p aria-live="polite">{status}</p>}
        <p className="note">
          Drag the cloud to turn it. Click a dot to see its file. This writes a
          graphify-out folder in the project you open. Password files are not
          opened. {graph.note}
        </p>
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
              </>
            ) : (
              <p>Click a dot in the cloud.</p>
            )}
          </aside>
        </div>
      </main>
    </>
  );
}

export default App;
