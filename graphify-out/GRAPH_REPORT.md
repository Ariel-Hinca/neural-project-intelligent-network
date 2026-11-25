# Graph Report - neural-project-intelligent-network  (2026-11-25)

## Corpus Check
- 20 files · ~6,965 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 7 file(s) not represented in the graph (top: .mdc 2, (none) 2, .icns 1)

## Summary
- 205 nodes · 316 edges · 18 communities (12 shown, 6 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `834e323f`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- NPIN stack
- check_fixture.py
- scan_manifests
- Graph
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
- Live

## God Nodes (most connected - your core abstractions)
1. `App()` - 19 edges
2. `compilerOptions` - 16 edges
3. `scan_manifests()` - 14 edges
4. `Found` - 11 edges
5. `Graph` - 11 edges
6. `keep_in_canon()` - 8 edges
7. `scan_roots()` - 7 edges
8. `scan_one()` - 7 edges
9. `canon_root()` - 6 edges
10. `apply_canon()` - 6 edges

## Surprising Connections (you probably didn't know these)
- `graph_from_manifests()` --references--> `Found`  [EXTRACTED]
  app/src-tauri/src/lib.rs → app/src-tauri/src/lib.rs  _Bridges community 2 → community 3_
- `attach_graphify()` --references--> `Graph`  [EXTRACTED]
  app/src-tauri/src/lib.rs → app/src-tauri/src/lib.rs  _Bridges community 3 → community 16_
- `LiveInner` --references--> `Graph`  [EXTRACTED]
  app/src-tauri/src/lib.rs → app/src-tauri/src/lib.rs  _Bridges community 3 → community 17_
- `scan_roots()` --references--> `AppHandle`  [EXTRACTED]
  app/src-tauri/src/lib.rs →   _Bridges community 3 → community 9_
- `diff_canon()` --calls--> `scan_manifests()`  [EXTRACTED]
  app/src-tauri/src/lib.rs → app/src-tauri/src/lib.rs  _Bridges community 2 → community 9_

## Import Cycles
- 1-file cycle: `app/src-tauri/src/lib.rs -> app/src-tauri/src/lib.rs`

## Communities (18 total, 6 thin omitted)

### Community 0 - "NPIN stack"
Cohesion: 0.40
Nodes (4): Decisions, Later phases, NPIN stack, Phase 0 install record

### Community 2 - "scan_manifests"
Cohesion: 0.26
Nodes (13): collect_skills(), Found, links_two_folders_through_one_shared_dot(), lists_an_agent_and_skips_scanner_output(), lists_packages_and_rules_without_reading_secrets(), read_agents(), read_api(), read_cargo_name() (+5 more)

### Community 3 - "Graph"
Cohesion: 0.31
Nodes (11): Graph, graph_from_manifests(), is_shared_kind(), Link, merge_all(), merge_graph(), Node, rescan_root() (+3 more)

### Community 4 - "package.json"
Cohesion: 0.07
Nodes (33): dependencies, react@^19.1.0, react-dom@^19.1.0, react-force-graph-3d@^1.29.2, @tauri-apps/api@^2, @tauri-apps/plugin-dialog@^2, devDependencies, @tauri-apps/cli@^2 (+25 more)

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
Cohesion: 0.13
Nodes (21): App(), addFolders(), compareFolder(), keepSelected(), load(), onConfirmClose(), openReport(), refreshCanon() (+13 more)

### Community 16 - "lib.rs"
Cohesion: 0.12
Nodes (5): attach_graphify(), find_skill(), find_skill_in(), graphify_command(), SCHEMA

### Community 17 - "Live"
Cohesion: 0.25
Nodes (6): arm_watch(), Live, LiveInner, path_is_noise(), plain_path(), root_for_change()

## Knowledge Gaps
- **1 isolated node(s):** `app`
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 88 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **6 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `App()` connect `App` to `package.json`?**
  _High betweenness centrality (0.030) - this node is a cross-community bridge._
- **What connects `app` to the rest of the system?**
  _1 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `package.json` be split into smaller, more focused modules?**
  _Cohesion score 0.07357357357357357 - nodes in this community are weakly interconnected._
- **Why does `react` connect `package.json` to `App`?**
  _High betweenness centrality (0.012) - this node is a cross-community bridge._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._
- **Should `tauri.conf.json` be split into smaller, more focused modules?**
  _Cohesion score 0.1111111111111111 - nodes in this community are weakly interconnected._
- **Should `App` be split into smaller, more focused modules?**
  _Cohesion score 0.13333333333333333 - nodes in this community are weakly interconnected._