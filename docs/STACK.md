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

Phase 0 and Phase 1 are complete. Phases 2 through 6 are in `app/`: add several folders, see one 3D cloud, keep skills and rules in a canon database, and rescan a folder when its files change. A package, skill, API, or database used by more than one folder is one shared dot. Agent files in `.cursor/agents` are dots. Compare a folder with the canon, then write missing skills and rules only after confirmation. The picture in `docs/npin-final-mockup.jpg` is an older flat sketch. The window opens when the Visual Studio desktop C++ tools are on PATH.

Phase 1 lives in `model/`: `schema.sql` (allowed kinds and links), `fixture.sql` (one hand-built example of this project), and `check_fixture.py` (loads that example into a temporary database). No window.

1. Model: done. Kinds, SQLite schema, one fixture graph. No window.
2. One repo: Tauri window, one folder, Graphify plus manifest scan, list view.
3. One graph: done in code. 3D cloud, filter by kind, click a dot to see its file.
4. God's eye: done in code. Several folders, one merged cloud. Shared package, skill, API, and database dots.
5. Shared memory: done in code. Canon vault in app data. Diff a folder. Apply missing skills and rules only after confirmation.
6. Living network: done in code. Open folders are watched, and a change rescans that folder. Agent markdown files in `.cursor/agents` are dots. Graphiti is not included. FalkorDB Lite rejects Windows, and Graphiti's embedded Relate driver was not verified here as a Windows-local store.
