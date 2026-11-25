# Graph Report - neural-project-intelligent-network  (2026-11-25)

## Corpus Check
- 19 files · ~5,887 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 7 file(s) not represented in the graph (top: .mdc 2, (none) 2, .icns 1)

## Summary
- 178 nodes · 256 edges · 16 communities (10 shown, 6 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `d5691844`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- NPIN stack
- check_fixture.py
- lib.rs
- App.tsx
- package.json
- compilerOptions
- tauri.conf.json
- compilerOptions
- default.json
- keep_in_canon
- Tauri + React + Typescript
- app
- App

## God Nodes (most connected - your core abstractions)
1. `compilerOptions` - 16 edges
2. `scan_manifests()` - 12 edges
3. `App()` - 12 edges
4. `Found` - 10 edges
5. `keep_in_canon()` - 8 edges
6. `Graph` - 7 edges
7. `scan_roots()` - 6 edges
8. `canon_root()` - 6 edges
9. `apply_canon()` - 6 edges
10. `compilerOptions` - 6 edges

## Surprising Connections (you probably didn't know these)
- `diff_canon()` --calls--> `scan_manifests()`  [EXTRACTED]
  app/src-tauri/src/lib.rs → app/src-tauri/src/lib.rs  _Bridges community 2 → community 9_

## Import Cycles
- 1-file cycle: `app/src-tauri/src/lib.rs -> app/src-tauri/src/lib.rs`

## Communities (16 total, 6 thin omitted)

### Community 0 - "NPIN stack"
Cohesion: 0.40
Nodes (4): Decisions, Later phases, NPIN stack, Phase 0 install record

### Community 2 - "lib.rs"
Cohesion: 0.12
Nodes (23): attach_graphify(), collect_skills(), find_skill(), find_skill_in(), Found, Graph, graph_from_manifests(), graphify_command() (+15 more)

### Community 3 - "App.tsx"
Cohesion: 0.13
Nodes (17): dependencies, react@^19.1.0, react-dom@^19.1.0, react-force-graph-3d@^1.29.2, @tauri-apps/api@^2, @tauri-apps/plugin-dialog@^2, CanonEntry, CanonGap (+9 more)

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

### Community 9 - "keep_in_canon"
Cohesion: 0.17
Nodes (18): apply_canon(), apply_canon_to_folder(), canon_list(), canon_root(), CanonEntry, CanonGap, CanonPick, copy_tree() (+10 more)

### Community 15 - "App"
Cohesion: 0.27
Nodes (9): App(), addFolders(), compareFolder(), keepSelected(), load(), onConfirmClose(), refreshCanon(), removeRoot() (+1 more)

## Knowledge Gaps
- **1 isolated node(s):** `app`
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 81 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **6 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `App()` connect `App` to `App.tsx`?**
  _High betweenness centrality (0.027) - this node is a cross-community bridge._
- **What connects `app` to the rest of the system?**
  _1 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `lib.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.12310606060606061 - nodes in this community are weakly interconnected._
- **Why does `react` connect `App.tsx` to `package.json`?**
  _High betweenness centrality (0.012) - this node is a cross-community bridge._
- **Should `App.tsx` be split into smaller, more focused modules?**
  _Cohesion score 0.13450292397660818 - nodes in this community are weakly interconnected._
- **Should `package.json` be split into smaller, more focused modules?**
  _Cohesion score 0.10333333333333333 - nodes in this community are weakly interconnected._
- **Should `compilerOptions` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._