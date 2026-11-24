use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Serialize)]
struct Found {
    kind: String,
    name: String,
    detail: String,
}

#[tauri::command]
fn scan_folder(path: String) -> Result<Vec<Found>, String> {
    let root = PathBuf::from(&path);
    let mut found = scan_manifests(&root)?;
    if let Some(repo) = found.first_mut() {
        let note = graphify_note(&root);
        repo.detail = format!("{} {note}", repo.detail);
    }
    Ok(found)
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
    read_rules(root, &mut found);
    read_skills(root, &mut found);
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

fn graphify_note(root: &Path) -> String {
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
    let count = value
        .get("nodes")
        .and_then(|nodes| nodes.as_array())
        .map(|nodes| nodes.len())
        .unwrap_or(0);
    format!("Graphify mapped {count} code things and wrote graphify-out in this folder.")
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![scan_folder])
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
        let text = format!("{found:?}");
        assert!(found.iter().any(|item| item.kind == "Package" && item.name == "react"));
        assert!(found.iter().any(|item| item.kind == "Rule" && item.name == "local.mdc"));
        assert!(!text.contains("super-secret-value"));

        fs::remove_dir_all(&root).unwrap();
    }
}
