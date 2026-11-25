use notify::Watcher;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{Emitter, Manager};

#[derive(Debug, Serialize)]
struct Found {
    kind: String,
    name: String,
    detail: String,
}

#[derive(Debug, Clone, Serialize)]
struct Node {
    id: String,
    kind: String,
    name: String,
    detail: String,
    file: String,
}

#[derive(Debug, Clone, Serialize)]
struct Link {
    source: String,
    target: String,
}

#[derive(Debug, Clone, Serialize)]
struct Graph {
    nodes: Vec<Node>,
    links: Vec<Link>,
    note: String,
}

#[tauri::command]
fn scan_roots(app: tauri::AppHandle, paths: Vec<String>) -> Result<Graph, String> {
    let unique = unique_paths(paths);
    if unique.is_empty() {
        return Err("Choose a folder.".into());
    }
    let mut parts = Vec::new();
    for path in &unique {
        parts.push(scan_one(path)?);
    }
    let merged = merge_all(&parts);
    let live = app.state::<Live>();
    {
        let mut guard = live.inner.lock().expect("watch");
        guard.roots = unique;
        guard.parts = parts;
        guard.dirty.clear();
    }
    arm_watch(&live);
    Ok(merged)
}

fn unique_paths(paths: Vec<String>) -> Vec<String> {
    let mut unique = Vec::new();
    for path in paths {
        if !unique.contains(&path) {
            unique.push(path);
        }
    }
    unique
}

fn scan_one(path: &str) -> Result<Graph, String> {
    let root = PathBuf::from(path);
    let found = scan_manifests(&root)?;
    let mut graph = graph_from_manifests(found, path);
    graph.note = attach_graphify(&root, &mut graph, path);
    Ok(graph)
}

fn merge_all(parts: &[Graph]) -> Graph {
    let mut merged = Graph {
        nodes: Vec::new(),
        links: Vec::new(),
        note: String::new(),
    };
    let mut notes = Vec::new();
    for part in parts {
        if !part.note.is_empty() {
            notes.push(part.note.clone());
        }
        merge_graph(&mut merged, part.clone());
    }
    merged.note = notes.join(" ");
    merged
}

fn is_shared_kind(kind: &str) -> bool {
    matches!(kind, "Package" | "Skill" | "API" | "Database")
}

fn graph_from_manifests(found: Vec<Found>, repo_path: &str) -> Graph {
    let repo_id = format!("repo:{repo_path}");
    let mut nodes = Vec::new();
    let mut links = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for (index, item) in found.into_iter().enumerate() {
        let id = if index == 0 {
            repo_id.clone()
        } else if is_shared_kind(&item.kind) {
            format!("{}:{}", item.kind, item.name)
        } else {
            format!("{repo_id}:{}:{}", item.kind, item.name)
        };
        if !seen.insert(id.clone()) {
            continue;
        }
        if index > 0 {
            links.push(Link {
                source: repo_id.clone(),
                target: id.clone(),
            });
        }
        nodes.push(Node {
            id,
            kind: item.kind.clone(),
            name: item.name.clone(),
            detail: item.detail,
            file: if item.kind == "Agent" {
                Path::new(repo_path)
                    .join(".cursor")
                    .join("agents")
                    .join(format!("{}.md", item.name))
                    .display()
                    .to_string()
            } else {
                String::new()
            },
        });
    }
    Graph {
        nodes,
        links,
        note: String::new(),
    }
}

fn merge_graph(into: &mut Graph, graph: Graph) {
    let mut have: std::collections::HashSet<String> =
        into.nodes.iter().map(|node| node.id.clone()).collect();
    for node in graph.nodes {
        if have.contains(&node.id) {
            if is_shared_kind(&node.kind) {
                if let Some(existing) = into.nodes.iter_mut().find(|item| item.id == node.id) {
                    if !existing.detail.starts_with("Shared across folders.") {
                        existing.detail = format!("Shared across folders. {}", existing.detail);
                    }
                }
            }
            continue;
        }
        have.insert(node.id.clone());
        into.nodes.push(node);
    }
    let mut seen: std::collections::HashSet<(String, String)> = into
        .links
        .iter()
        .map(|link| (link.source.clone(), link.target.clone()))
        .collect();
    for link in graph.links {
        if seen.insert((link.source.clone(), link.target.clone())) {
            into.links.push(link);
        }
    }
}

fn scan_manifests(root: &Path) -> Result<Vec<Found>, String> {
    if !root.is_dir() {
        return Err("Choose a folder.".into());
    }
    let mut found = Vec::new();
    let folder_name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("folder");
    found.push(Found {
        kind: "Repository".into(),
        name: folder_name.to_string(),
        detail: root.display().to_string(),
    });
    read_package_json(root, &mut found);
    read_cargo_name(root, &mut found);
    read_api(root, &mut found);
    read_database(root, &mut found);
    read_rules(root, &mut found);
    read_skills(root, &mut found);
    read_agents(root, &mut found);
    if root.join("AGENTS.md").is_file() {
        found.push(Found {
            kind: "Rule".into(),
            name: "AGENTS.md".into(),
            detail: "Agent instructions in this folder.".into(),
        });
    }
    Ok(found)
}

fn read_package_json(root: &Path, found: &mut Vec<Found>) {
    let Ok(text) = fs::read_to_string(root.join("package.json")) else {
        return;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return;
    };
    for key in ["dependencies", "devDependencies"] {
        let Some(deps) = value.get(key).and_then(|item| item.as_object()) else {
            continue;
        };
        for name in deps.keys() {
            found.push(Found {
                kind: "Package".into(),
                name: name.clone(),
                detail: format!("Listed in package.json {key}."),
            });
        }
    }
}

fn read_cargo_name(root: &Path, found: &mut Vec<Found>) {
    let Ok(text) = fs::read_to_string(root.join("Cargo.toml")) else {
        return;
    };
    let mut in_package = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if in_package {
            if let Some(name) = trimmed.strip_prefix("name = ") {
                let name = name.trim().trim_matches('"');
                if !name.is_empty() {
                    found.push(Found {
                        kind: "Package".into(),
                        name: name.to_string(),
                        detail: "Named in Cargo.toml.".into(),
                    });
                }
                break;
            }
        }
    }
}

// shortcut: openapi.json and schema.prisma at the folder root only, upgrade when those files live deeper
fn read_api(root: &Path, found: &mut Vec<Found>) {
    let Ok(text) = fs::read_to_string(root.join("openapi.json")) else {
        return;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return;
    };
    let Some(title) = value
        .get("info")
        .and_then(|info| info.get("title"))
        .and_then(|title| title.as_str())
    else {
        return;
    };
    if title.is_empty() {
        return;
    }
    found.push(Found {
        kind: "API".into(),
        name: title.to_string(),
        detail: "Named in openapi.json.".into(),
    });
}

fn read_database(root: &Path, found: &mut Vec<Found>) {
    let Ok(text) = fs::read_to_string(root.join("schema.prisma")) else {
        return;
    };
    let mut name = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("datasource ") {
            let name_text = rest.trim().trim_end_matches('{').trim();
            if !name_text.is_empty() {
                name = Some(name_text.to_string());
            }
            break;
        }
    }
    let Some(name) = name else {
        return;
    };
    found.push(Found {
        kind: "Database".into(),
        name,
        detail: "Named in schema.prisma.".into(),
    });
}

fn read_rules(root: &Path, found: &mut Vec<Found>) {
    let Ok(entries) = fs::read_dir(root.join(".cursor").join("rules")) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".mdc") || name.ends_with(".md") {
            found.push(Found {
                kind: "Rule".into(),
                name,
                detail: "Cursor rule in this folder.".into(),
            });
        }
    }
}

fn read_skills(root: &Path, found: &mut Vec<Found>) {
    for base in [
        root.join(".cursor").join("skills"),
        root.join(".agents").join("skills"),
    ] {
        collect_skills(&base, found, 0);
    }
}

fn collect_skills(dir: &Path, found: &mut Vec<Found>, depth: u8) {
    if depth > 3 {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if path.join("SKILL.md").is_file() {
            found.push(Found {
                kind: "Skill".into(),
                name,
                detail: "Skill folder in this project.".into(),
            });
        } else {
            collect_skills(&path, found, depth + 1);
        }
    }
}

// shortcut: only markdown files directly inside .cursor/agents, upgrade when a repo nests agent files
fn read_agents(root: &Path, found: &mut Vec<Found>) {
    let Ok(entries) = fs::read_dir(root.join(".cursor").join("agents")) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let filename = entry.file_name().to_string_lossy().to_string();
        let Some(name) = filename.strip_suffix(".md") else {
            continue;
        };
        if name.is_empty() {
            continue;
        }
        found.push(Found {
            kind: "Agent".into(),
            name: name.to_string(),
            detail: "Cursor agent in this folder.".into(),
        });
    }
}

fn attach_graphify(root: &Path, graph: &mut Graph, repo_path: &str) -> String {
    let mut command = graphify_command();
    let output = command.arg("update").arg(".").current_dir(root).output();
    let output = match output {
        Ok(output) => output,
        Err(error) => return format!("Graphify is not available ({error})."),
    };
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let line = err.lines().last().unwrap_or("Graphify failed.");
        return format!("Graphify did not finish: {line}");
    }
    let graph_path = root.join("graphify-out").join("graph.json");
    let Ok(text) = fs::read_to_string(&graph_path) else {
        return "Graphify finished, but graph.json is missing.".into();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return "Graphify wrote a graph file that could not be read.".into();
    };
    let Some(raw_nodes) = value.get("nodes").and_then(|nodes| nodes.as_array()) else {
        return "Graphify wrote a graph with no dots.".into();
    };
    // shortcut: first 400 code dots, upgrade when a big folder needs a sampled cloud
    let mut kept = std::collections::HashSet::new();
    for raw in raw_nodes.iter().take(400) {
        let Some(id) = raw.get("id").and_then(|item| item.as_str()) else {
            continue;
        };
        let file = raw
            .get("source_file")
            .and_then(|item| item.as_str())
            .unwrap_or("");
        if file.contains(".env") {
            continue;
        }
        let name = raw
            .get("label")
            .and_then(|item| item.as_str())
            .unwrap_or(id);
        let kind = match raw.get("file_type").and_then(|item| item.as_str()) {
            Some("document") => "Document",
            Some("rationale") => "Note",
            _ => "Code",
        };
        let node_id = format!("g:{repo_path}:{id}");
        kept.insert(node_id.clone());
        graph.nodes.push(Node {
            id: node_id,
            kind: kind.into(),
            name: name.to_string(),
            detail: "From the local code map.".into(),
            file: file.to_string(),
        });
    }
    if let Some(raw_links) = value.get("links").and_then(|links| links.as_array()) {
        for raw in raw_links {
            let Some(source) = raw.get("source").and_then(|item| item.as_str()) else {
                continue;
            };
            let Some(target) = raw.get("target").and_then(|item| item.as_str()) else {
                continue;
            };
            let source = format!("g:{repo_path}:{source}");
            let target = format!("g:{repo_path}:{target}");
            if kept.contains(&source) && kept.contains(&target) {
                graph.links.push(Link { source, target });
            }
        }
    }
    format!(
        "Graphify mapped {} code things. This cloud shows {} of them and wrote graphify-out in this folder.",
        raw_nodes.len(),
        kept.len()
    )
}

// shortcut: also looks in the user .local/bin, upgrade when the app stores the scanner path
fn graphify_command() -> Command {
    if let Ok(profile) = std::env::var("USERPROFILE") {
        let exe = PathBuf::from(profile)
            .join(".local")
            .join("bin")
            .join("graphify.exe");
        if exe.is_file() {
            return Command::new(exe);
        }
    }
    Command::new("graphify")
}

#[derive(Debug, Serialize)]
struct CanonEntry {
    kind: String,
    name: String,
}

#[derive(Debug, Serialize)]
struct CanonGap {
    kind: String,
    name: String,
    missing: bool,
}

#[derive(Debug, Deserialize)]
struct CanonPick {
    kind: String,
    name: String,
}

const SCHEMA: &str = include_str!("../../../model/schema.sql");

fn canon_root(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let base = app.path().app_data_dir().map_err(|error| error.to_string())?;
    let dir = base.join("canon");
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    Ok(dir)
}

fn open_canon(dir: &Path) -> Result<rusqlite::Connection, String> {
    fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let path = dir.join("canon.sqlite");
    let fresh = !path.is_file();
    let conn = rusqlite::Connection::open(&path).map_err(|error| error.to_string())?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|error| error.to_string())?;
    if fresh {
        conn.execute_batch(SCHEMA).map_err(|error| error.to_string())?;
    }
    Ok(conn)
}

fn ensure_canon_repo(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute(
        "INSERT OR IGNORE INTO nodes (id, kind, name, detail) VALUES ('repo:canon', 'Repository', 'canon', 'Shared memory for skills and rules.')",
        [],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn safe_canon_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.starts_with('.')
        || name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|'])
        || name.contains("..")
    {
        return Err("That name cannot be stored.".into());
    }
    Ok(())
}

fn is_secret_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower == ".env"
        || lower.starts_with(".env.")
        || lower.ends_with(".pem")
        || lower.ends_with(".key")
        || lower == "credentials.json"
}

fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|error| error.to_string())?;
    for entry in fs::read_dir(from).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let meta = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if meta.file_type().is_symlink() {
            continue;
        }
        let name = entry.file_name();
        if is_secret_name(&name.to_string_lossy()) {
            continue;
        }
        let dest = to.join(&name);
        if meta.is_dir() {
            copy_tree(&path, &dest)?;
        } else if meta.is_file() {
            fs::copy(&path, &dest).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

// shortcut: a skill must be a direct or nested folder under .cursor/skills or .agents/skills, upgrade when a repo stores skills elsewhere
fn find_skill(root: &Path, name: &str) -> Option<PathBuf> {
    for base in [
        root.join(".cursor").join("skills"),
        root.join(".agents").join("skills"),
    ] {
        if let Some(found) = find_skill_in(&base, name, 0) {
            return Some(found);
        }
    }
    None
}

fn find_skill_in(dir: &Path, name: &str, depth: u8) -> Option<PathBuf> {
    if depth > 3 {
        return None;
    }
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(meta) = fs::symlink_metadata(&path) else {
            continue;
        };
        if meta.file_type().is_symlink() || !meta.is_dir() {
            continue;
        }
        let folder = entry.file_name().to_string_lossy().to_string();
        if folder == name && path.join("SKILL.md").is_file() {
            return Some(path);
        }
        if let Some(found) = find_skill_in(&path, name, depth + 1) {
            return Some(found);
        }
    }
    None
}

fn remember_node(conn: &rusqlite::Connection, kind: &str, name: &str) -> Result<(), String> {
    let id = format!("{}:{name}", kind.to_ascii_lowercase());
    conn.execute(
        "INSERT OR IGNORE INTO nodes (id, kind, name, detail) VALUES (?1, ?2, ?3, 'Kept from a folder.')",
        rusqlite::params![id, kind, name],
    )
    .map_err(|error| error.to_string())?;
    conn.execute(
        "INSERT OR IGNORE INTO edges (from_id, to_id, relation, origin) VALUES ('repo:canon', ?1, 'contains', 'canon')",
        rusqlite::params![id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn keep_in_canon(dir: &Path, root: &Path, kind: &str, name: &str) -> Result<String, String> {
    if !root.is_dir() {
        return Err("Choose a folder.".into());
    }
    safe_canon_name(name)?;
    match kind {
        "Skill" => {
            let Some(source) = find_skill(root, name) else {
                return Err(format!("No skill named {name} in that folder."));
            };
            let dest = dir.join("files").join("skills").join(name);
            if dest.exists() {
                fs::remove_dir_all(&dest).map_err(|error| error.to_string())?;
            }
            copy_tree(&source, &dest)?;
        }
        "Rule" => {
            if !name.ends_with(".md") && !name.ends_with(".mdc") {
                return Err("A rule must be a markdown file.".into());
            }
            let source = root.join(".cursor").join("rules").join(name);
            if !source.is_file() {
                return Err(format!("No rule named {name} in that folder."));
            }
            let dest_dir = dir.join("files").join("rules");
            fs::create_dir_all(&dest_dir).map_err(|error| error.to_string())?;
            fs::copy(&source, dest_dir.join(name)).map_err(|error| error.to_string())?;
        }
        _ => return Err("Only a skill or a rule can be kept.".into()),
    }
    let conn = open_canon(dir)?;
    ensure_canon_repo(&conn)?;
    remember_node(&conn, kind, name)?;
    Ok(format!("Saved {name}."))
}

fn list_canon(dir: &Path) -> Result<Vec<CanonEntry>, String> {
    let conn = open_canon(dir)?;
    let mut stmt = conn
        .prepare("SELECT kind, name FROM nodes WHERE kind IN ('Skill', 'Rule') ORDER BY kind, name")
        .map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(CanonEntry {
                kind: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .map_err(|error| error.to_string())?;
    let mut entries = Vec::new();
    for row in rows {
        entries.push(row.map_err(|error| error.to_string())?);
    }
    Ok(entries)
}

fn diff_canon(dir: &Path, root: &Path) -> Result<Vec<CanonGap>, String> {
    let found = scan_manifests(root)?;
    let mut gaps = Vec::new();
    for entry in list_canon(dir)? {
        let missing = !found
            .iter()
            .any(|item| item.kind == entry.kind && item.name == entry.name);
        gaps.push(CanonGap {
            kind: entry.kind,
            name: entry.name,
            missing,
        });
    }
    Ok(gaps)
}

fn apply_canon(
    dir: &Path,
    root: &Path,
    picks: &[CanonPick],
    confirmed: bool,
) -> Result<String, String> {
    if !confirmed {
        return Err("Confirm this folder before writing skills or rules into it.".into());
    }
    if !root.is_dir() {
        return Err("Choose a folder.".into());
    }
    if picks.is_empty() {
        return Err("Choose a missing skill or rule.".into());
    }
    let conn = open_canon(dir)?;
    let mut wrote = Vec::new();
    let mut skipped = Vec::new();
    for pick in picks {
        safe_canon_name(&pick.name)?;
        let stored: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM nodes WHERE kind = ?1 AND name = ?2",
                rusqlite::params![pick.kind, pick.name],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;
        if stored != 1 {
            return Err(format!("Nothing saved named {}.", pick.name));
        }
        match pick.kind.as_str() {
            "Skill" => {
                let dest = root.join(".cursor").join("skills").join(&pick.name);
                if dest.exists() {
                    skipped.push(pick.name.clone());
                    continue;
                }
                let source = dir.join("files").join("skills").join(&pick.name);
                if !source.join("SKILL.md").is_file() {
                    return Err(format!("No saved files for {}.", pick.name));
                }
                copy_tree(&source, &dest)?;
                wrote.push(pick.name.clone());
            }
            "Rule" => {
                let dest = root.join(".cursor").join("rules").join(&pick.name);
                if dest.exists() {
                    skipped.push(pick.name.clone());
                    continue;
                }
                let source = dir.join("files").join("rules").join(&pick.name);
                if !source.is_file() {
                    return Err(format!("No saved files for {}.", pick.name));
                }
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
                }
                fs::copy(&source, &dest).map_err(|error| error.to_string())?;
                wrote.push(pick.name.clone());
            }
            _ => return Err("Only a skill or a rule can be written.".into()),
        }
    }
    Ok(format!(
        "Wrote {}. Left existing files in place: {}.",
        if wrote.is_empty() {
            "nothing".to_string()
        } else {
            wrote.join(", ")
        },
        if skipped.is_empty() {
            "none".to_string()
        } else {
            skipped.join(", ")
        }
    ))
}

#[tauri::command]
fn canon_list(app: tauri::AppHandle) -> Result<Vec<CanonEntry>, String> {
    list_canon(&canon_root(&app)?)
}

#[tauri::command]
fn keep_canon(app: tauri::AppHandle, root: String, kind: String, name: String) -> Result<String, String> {
    keep_in_canon(&canon_root(&app)?, Path::new(&root), &kind, &name)
}

#[tauri::command]
fn diff_against_canon(app: tauri::AppHandle, root: String) -> Result<Vec<CanonGap>, String> {
    diff_canon(&canon_root(&app)?, Path::new(&root))
}

#[tauri::command]
fn apply_canon_to_folder(
    app: tauri::AppHandle,
    root: String,
    picks: Vec<CanonPick>,
    confirmed: bool,
) -> Result<String, String> {
    apply_canon(&canon_root(&app)?, Path::new(&root), &picks, confirmed)
}

struct LiveInner {
    roots: Vec<String>,
    parts: Vec<Graph>,
    dirty: std::collections::HashMap<String, Instant>,
}

struct Live {
    inner: Arc<Mutex<LiveInner>>,
    watcher: Mutex<Option<notify::RecommendedWatcher>>,
}

// shortcut: noisy folders are filtered after the event, upgrade when a large folder should not be watched recursively
fn path_is_noise(path: &Path) -> bool {
    path.components().any(|component| {
        let text = component.as_os_str().to_string_lossy();
        matches!(
            text.as_ref(),
            "node_modules" | "graphify-out" | "target" | ".git" | "dist"
        )
    })
}

fn root_for_change(roots: &[String], path: &Path) -> Option<String> {
    let path = plain_path(path);
    let path = Path::new(&path);
    if path_is_noise(path) {
        return None;
    }
    roots
        .iter()
        .filter(|root| path.starts_with(plain_path(Path::new(root))))
        .max_by_key(|root| root.len())
        .cloned()
}

fn plain_path(path: &Path) -> String {
    let text = path.to_string_lossy();
    text.strip_prefix(r"\\?\").unwrap_or(&text).to_string()
}

fn arm_watch(live: &Live) {
    let roots = live.inner.lock().expect("watch").roots.clone();
    let inner = Arc::clone(&live.inner);
    let Ok(mut watcher) = notify::recommended_watcher(move |result: Result<notify::Event, notify::Error>| {
        let Ok(event) = result else {
            return;
        };
        let mut guard = inner.lock().expect("watch");
        for path in event.paths {
            if let Some(root) = root_for_change(&guard.roots, &path) {
                guard.dirty.insert(root, Instant::now());
            }
        }
    }) else {
        return;
    };
    for root in &roots {
        let _ = watcher.watch(Path::new(root), notify::RecursiveMode::Recursive);
    }
    *live.watcher.lock().expect("watch") = Some(watcher);
}

#[tauri::command]
fn rescan_root(app: tauri::AppHandle, path: String) -> Result<Graph, String> {
    let part = scan_one(&path)?;
    let live = app.state::<Live>();
    let mut guard = live.inner.lock().expect("watch");
    let Some(index) = guard.roots.iter().position(|root| root == &path) else {
        return Err("That folder is not open.".into());
    };
    guard.parts[index] = part;
    Ok(merge_all(&guard.parts))
}

#[tauri::command]
fn clear_watch(app: tauri::AppHandle) {
    let live = app.state::<Live>();
    {
        let mut guard = live.inner.lock().expect("watch");
        guard.roots.clear();
        guard.parts.clear();
        guard.dirty.clear();
    }
    *live.watcher.lock().expect("watch") = None;
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let inner = Arc::new(Mutex::new(LiveInner {
        roots: Vec::new(),
        parts: Vec::new(),
        dirty: std::collections::HashMap::new(),
    }));
    let pump = Arc::clone(&inner);
    tauri::Builder::default()
        .manage(Live {
            inner,
            watcher: Mutex::new(None),
        })
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            let handle = app.handle().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_millis(400));
                let ready = {
                    let mut guard = pump.lock().expect("watch");
                    let now = Instant::now();
                    let ready: Vec<String> = guard
                        .dirty
                        .iter()
                        .filter(|(_, seen)| now.duration_since(**seen) >= Duration::from_millis(800))
                        .map(|(root, _)| root.clone())
                        .collect();
                    for root in &ready {
                        guard.dirty.remove(root);
                    }
                    ready
                };
                for root in ready {
                    let _ = handle.emit("root-changed", root);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_roots,
            rescan_root,
            clear_watch,
            canon_list,
            keep_canon,
            diff_against_canon,
            apply_canon_to_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::scan_manifests;
    use std::fs;

    #[test]
    fn lists_packages_and_rules_without_reading_secrets() {
        let root = std::env::temp_dir().join(format!("npin-scan-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".cursor").join("rules")).unwrap();
        fs::write(
            root.join("package.json"),
            r#"{"dependencies":{"react":"19.0.0"}}"#,
        )
        .unwrap();
        fs::write(root.join(".env"), "PASSWORD=super-secret-value").unwrap();
        fs::write(root.join(".cursor").join("rules").join("local.mdc"), "# rule").unwrap();

        let found = scan_manifests(&root).unwrap();
        let repo_path = root.display().to_string();
        let graph = super::graph_from_manifests(found, &repo_path);
        let text = format!("{graph:?}");
        assert!(graph.nodes.iter().any(|item| item.kind == "Package" && item.name == "react"));
        assert!(graph.nodes.iter().any(|item| item.kind == "Rule" && item.name == "local.mdc"));
        assert!(graph.links.iter().any(|link| link.source == format!("repo:{repo_path}")));
        assert!(!text.contains("super-secret-value"));

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn links_two_folders_through_one_shared_dot() {
        let base = std::env::temp_dir().join(format!("npin-merge-{}", std::process::id()));
        let left = base.join("left");
        let right = base.join("right");
        let _ = fs::remove_dir_all(&base);
        for root in [&left, &right] {
            let skill = root.join(".agents").join("skills").join("ponytail");
            fs::create_dir_all(&skill).unwrap();
            fs::write(skill.join("SKILL.md"), "# skill").unwrap();
            fs::write(root.join("package.json"), r#"{"dependencies":{"react":"1.0.0"}}"#).unwrap();
            fs::write(root.join("openapi.json"), r#"{"info":{"title":"Billing"}}"#).unwrap();
            fs::write(
                root.join("schema.prisma"),
                "datasource canon {\n  provider = \"sqlite\"\n}\n",
            )
            .unwrap();
            fs::create_dir_all(root.join(".cursor").join("rules")).unwrap();
        }
        fs::write(left.join(".cursor").join("rules").join("left.mdc"), "# left").unwrap();
        fs::write(right.join(".cursor").join("rules").join("right.mdc"), "# right").unwrap();

        let mut merged = super::graph_from_manifests(
            scan_manifests(&left).unwrap(),
            &left.display().to_string(),
        );
        super::merge_graph(
            &mut merged,
            super::graph_from_manifests(scan_manifests(&right).unwrap(), &right.display().to_string()),
        );

        for (kind, name) in [
            ("Package", "react"),
            ("Skill", "ponytail"),
            ("API", "Billing"),
            ("Database", "canon"),
        ] {
            let shared: Vec<_> = merged
                .nodes
                .iter()
                .filter(|node| node.kind == kind && node.name == name)
                .collect();
            assert_eq!(shared.len(), 1, "{kind} {name}");
            assert!(shared[0].detail.starts_with("Shared across folders."));
            assert_eq!(
                merged.links.iter().filter(|link| link.target == shared[0].id).count(),
                2
            );
        }
        assert_eq!(merged.nodes.iter().filter(|node| node.kind == "Repository").count(), 2);
        assert_eq!(merged.nodes.iter().filter(|node| node.kind == "Rule").count(), 2);

        fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn writes_missing_canon_files_only_after_confirmation() {
        let base = std::env::temp_dir().join(format!("npin-canon-{}", std::process::id()));
        let canon = base.join("canon");
        let source = base.join("source");
        let target = base.join("target");
        let _ = fs::remove_dir_all(&base);
        let skill = source.join(".agents").join("skills").join("ponytail");
        fs::create_dir_all(&skill).unwrap();
        fs::write(skill.join("SKILL.md"), "# skill").unwrap();
        fs::write(skill.join(".env"), "PASSWORD=super-secret-value").unwrap();
        fs::create_dir_all(source.join(".cursor").join("rules")).unwrap();
        fs::write(source.join(".cursor").join("rules").join("local.mdc"), "# rule").unwrap();
        fs::create_dir_all(&target).unwrap();

        assert!(super::keep_in_canon(&canon, &source, "Skill", "ponytail").is_ok());
        assert!(super::keep_in_canon(&canon, &source, "Rule", "local.mdc").is_ok());
        assert!(!canon.join("files").join("skills").join("ponytail").join(".env").exists());

        let gaps = super::diff_canon(&canon, &target).unwrap();
        assert_eq!(gaps.len(), 2);
        assert!(gaps.iter().all(|gap| gap.missing));

        let picks = vec![
            super::CanonPick { kind: "Skill".into(), name: "ponytail".into() },
            super::CanonPick { kind: "Rule".into(), name: "local.mdc".into() },
        ];
        let refused = super::apply_canon(&canon, &target, &picks, false).unwrap_err();
        assert!(refused.contains("Confirm"));
        assert!(!target.join(".cursor").join("skills").join("ponytail").join("SKILL.md").exists());

        let wrote = super::apply_canon(&canon, &target, &picks, true).unwrap();
        assert!(wrote.contains("ponytail"));
        assert!(target.join(".cursor").join("skills").join("ponytail").join("SKILL.md").is_file());
        assert!(!target.join(".cursor").join("skills").join("ponytail").join(".env").exists());
        assert!(target.join(".cursor").join("rules").join("local.mdc").is_file());

        let again = super::apply_canon(&canon, &target, &picks, true).unwrap();
        assert!(again.contains("Left existing files"));
        let after = super::diff_canon(&canon, &target).unwrap();
        assert!(after.iter().all(|gap| !gap.missing));

        fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn lists_an_agent_and_skips_scanner_output() {
        let root = std::env::temp_dir().join(format!("npin-agent-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".cursor").join("agents")).unwrap();
        fs::write(root.join(".cursor").join("agents").join("reviewer.md"), "# reviewer").unwrap();
        fs::write(
            root.join(".cursor").join("agents").join(".env"),
            "PASSWORD=super-secret-value",
        )
        .unwrap();

        let found = scan_manifests(&root).unwrap();
        let repo_path = root.display().to_string();
        let graph = super::graph_from_manifests(found, &repo_path);
        let agent = graph.nodes.iter().find(|node| node.kind == "Agent").unwrap();
        assert_eq!(agent.name, "reviewer");
        assert!(agent.file.ends_with("reviewer.md"));
        assert!(!format!("{graph:?}").contains("super-secret-value"));

        let roots = vec![repo_path.clone()];
        assert_eq!(
            super::root_for_change(&roots, &root.join("src").join("main.rs")).as_deref(),
            Some(repo_path.as_str())
        );
        assert!(super::root_for_change(&roots, &root.join("graphify-out").join("graph.json")).is_none());

        fs::remove_dir_all(&root).unwrap();
    }
}
