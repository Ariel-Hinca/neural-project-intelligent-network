# NPIN stack

NPIN (Neural Project Intelligent Network) is a local Windows desktop app: a God's-eye graph of chosen repositories, plus a shared memory new projects can inherit.

## Decisions

- App shell: Tauri 2, React, TypeScript, Vite.
- Code graph: local Graphify CLI (`graphifyy` 0.9.118). Per-repo output is `graphify-out/graph.json`. NPIN reads that file and does not reimplement parsing.
- Canon store: one SQLite file in app data. No Neo4j and no Docker.
- Graph view: a 3D cloud (`react-force-graph-3d`), in Phase 3. Drag to turn it. A flat map was rejected.
- Graphiti is not v1 memory. The embedded FalkorDB Lite path is not a Windows target. Revisit only in Phase 6 if a Windows-local backend is verified.
- Electron is a fallback only if Tauri blocks file access or the graph view.

Node kinds: Repository, Service, API, Database, Package, Skill, Agent, Rule, Concept. Edges are extracted from code and Cursor files, or canon decisions kept on purpose. Inheritance writes into a repo only after that repo is confirmed. Secret files are never read. See `.cursor/rules/npin-secrets.mdc`.

## Phase 0 install record

Marketplace plugins:

- Context7: installed.
- Modern Web Guidance: installed.
- Hosted Graphify: declined. Local CLI remains the scanner.

Global Cursor skills (user scope, `npx skills add -a cursor -g`):

- DietrichGebert/ponytail
- mattpocock/skills
- tt-a1i/archify
- pbakaus/impeccable
- nextlevelbuilder/ui-ux-pro-max-skill
- Leonxlnx/taste-skill
- cathrynlavery/diagram-design
- Panniantong/Agent-Reach
- GoogleChrome/modern-web-guidance
- euxaristia/roast-me (skill name `roast-me`; the GitHub repo is archived and still the published install source)

Sequential Thinking is the official MCP server `@modelcontextprotocol/server-sequential-thinking` in the user `mcp.json`. It is not a skill folder.

Local scanner: `uv` 0.13.9, `graphify` 0.9.118 on the user PATH (`uv tool update-shell`). `graphify cursor install` wrote `.cursor/rules/graphify.mdc`. `graphify-out/graph.json` is not built yet; that happens when a repository is scanned in a later phase.

## Later phases

Phase 0 and Phase 1 are complete. Phase 2 and Phase 3 are in `app/`: open one folder and see a 3D cloud. The picture in `docs/npin-final-mockup.jpg` is an older flat sketch. The window has not been opened yet: Rust is installed, but the C++ libraries used to compile it are not. The next sprint starts only when asked.

Phase 1 lives in `model/`: `schema.sql` (allowed kinds and links), `fixture.sql` (one hand-built example of this project), and `check_fixture.py` (loads that example into a temporary database). No window.

1. Model: done. Kinds, SQLite schema, one fixture graph. No window.
2. One repo: Tauri window, one folder, Graphify plus manifest scan, list view.
3. One graph: done in code. 3D cloud for that repo, filter by kind, click a dot to see its file. The window still needs the C++ libraries before it can open.
4. God's eye: several roots, one merged graph.
5. Shared memory: canon vault, diff, confirmed apply.
6. Living network: watch and rescan. Agents as nodes. Graphiti only if Windows-local storage checks out.
