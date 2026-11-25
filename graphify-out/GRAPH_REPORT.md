# Graph Report - neural-project-intelligent-network  (2026-11-25)

## Corpus Check
- 20 files · ~6,959 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 7 file(s) not represented in the graph (top: .mdc 2, (none) 2, .icns 1)

## Summary
- 203 nodes · 310 edges · 17 communities (11 shown, 6 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `07523758`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- NPIN stack
- check_fixture.py
- scan_manifests
- App.tsx
- package.json
- compilerOptions
- tauri.conf.json
- compilerOptions
- default.json
- lib.rs
- Tauri + React + Typescript
- app
- App
- lib.rs

## God Nodes (most connected - your core abstractions)
1. `App()` - 17 edges
2. `compilerOptions` - 16 edges
3. `scan_manifests()` - 14 edges
4. `Graph` - 11 edges
5. `Found` - 11 edges
6. `keep_in_canon()` - 8 edges
7. `scan_one()` - 7 edges
8. `scan_roots()` - 7 edges
9. `compilerOptions` - 6 edges
10. `apply_canon()` - 6 edges

## Surprising Connections (you probably didn't know these)
- `App()` --calls--> `folderName()`  [EXTRACTED]
  app/src/App.tsx → app/src/App.tsx  _Bridges community 3 → community 15_
- `graph_from_manifests()` --references--> `Found`  [EXTRACTED]
  app/src-tauri/src/lib.rs → app/src-tauri/src/lib.rs  _Bridges community 2 → community 16_
- `rescan_root()` --references--> `AppHandle`  [EXTRACTED]
  app/src-tauri/src/lib.rs →   _Bridges community 16 → community 9_
- `diff_canon()` --calls--> `scan_manifests()`  [EXTRACTED]
  app/src-tauri/src/lib.rs → app/src-tauri/src/lib.rs  _Bridges community 2 → community 9_

## Import Cycles
- 1-file cycle: `app/src-tauri/src/lib.rs -> app/src-tauri/src/lib.rs`

## Communities (17 total, 6 thin omitted)

### Community 0 - "NPIN stack"
Cohesion: 0.40
Nodes (4): Decisions, Later phases, NPIN stack, Phase 0 install record

### Community 2 - "scan_manifests"
Cohesion: 0.26
Nodes (13): collect_skills(), Found, links_two_folders_through_one_shared_dot(), lists_an_agent_and_skips_scanner_output(), lists_packages_and_rules_without_reading_secrets(), read_agents(), read_api(), read_cargo_name() (+5 more)

### Community 3 - "App.tsx"
Cohesion: 0.12
Nodes (19): dependencies, react@^19.1.0, react-dom@^19.1.0, react-force-graph-3d@^1.29.2, @tauri-apps/api@^2, @tauri-apps/plugin-dialog@^2, CanonEntry, CanonGap (+11 more)

### Community 4 - "package.json"
Cohesion: 0.10
Nodes (22): devDependencies, @tauri-apps/cli@^2, @types/react@^19.1.8, @types/react-dom@^19.1.6, typescript@~6.0.3, vite@^8.0.16, @vitejs/plugin-react@^6.0.2, name (+14 more)

### Community 5 - "compilerOptions"
Cohesion: 0.11
Nodes (18): compilerOptions, allowImportingTsExtensions, isolatedModules, jsx, lib, module, moduleResolution, noEmit (+10 more)

### Community 6 - "tauri.conf.json"
Cohesion: 0.11
Nodes (17): app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, frontendDist (+9 more)

### Community 7 - "compilerOptions"
Cohesion: 0.25
Nodes (7): compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck, include

### Community 8 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 9 - "lib.rs"
Cohesion: 0.16
Nodes (19): apply_canon(), apply_canon_to_folder(), canon_list(), canon_root(), CanonEntry, CanonGap, CanonPick, clear_watch() (+11 more)

### Community 15 - "App"
Cohesion: 0.24
Nodes (11): App(), addFolders(), compareFolder(), keepSelected(), load(), onConfirmClose(), openReport(), refreshCanon() (+3 more)

### Community 16 - "lib.rs"
Cohesion: 0.09
Nodes (22): arm_watch(), attach_graphify(), find_skill(), find_skill_in(), Graph, graph_from_manifests(), graphify_command(), is_shared_kind() (+14 more)

## Knowledge Gaps
- **1 isolated node(s):** `app`
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 88 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **6 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `App()` connect `App` to `App.tsx`?**
  _High betweenness centrality (0.030) - this node is a cross-community bridge._
- **What connects `app` to the rest of the system?**
  _1 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `App.tsx` be split into smaller, more focused modules?**
  _Cohesion score 0.11904761904761904 - nodes in this community are weakly interconnected._
- **Why does `react` connect `App.tsx` to `package.json`?**
  _High betweenness centrality (0.011) - this node is a cross-community bridge._
- **Should `package.json` be split into smaller, more focused modules?**
  _Cohesion score 0.10333333333333333 - nodes in this community are weakly interconnected._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._
- **Should `tauri.conf.json` be split into smaller, more focused modules?**
  _Cohesion score 0.1111111111111111 - nodes in this community are weakly interconnected._