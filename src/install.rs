use std::fs;
use std::path::PathBuf;

// ── Canonical sources ─────────────────────────────────────────────────────

static SKILL_FETCH: &str = include_str!("../skills/obscura-fetch/SKILL.md");
static SKILL_SCRAPE: &str = include_str!("../skills/obscura-scrape/SKILL.md");
static SKILL_PIPELINE: &str = include_str!("../skills/obscura-pipeline/SKILL.md");

static CANONICAL_SKILLS: &[(&str, &str)] = &[
    ("obscura-fetch", SKILL_FETCH),
    ("obscura-scrape", SKILL_SCRAPE),
    ("obscura-pipeline", SKILL_PIPELINE),
];

// ── Tool registry ────────────────────────────────────────────────────────

pub static ALL_TOOLS: &[(&str, &str)] = &[
    ("cursor", "Cursor"),
    ("opencode", "OpenCode"),
    ("cline", "Cline"),
];

// ── Tool config ──────────────────────────────────────────────────────────

struct ToolConfig {
    name: &'static str,
    skills_dir: Box<dyn Fn(&str) -> PathBuf>,
}

fn home() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"))
}

fn tool_config(id: &str) -> Option<ToolConfig> {
    match id {
        "cursor" => Some(ToolConfig {
            name: "Cursor",
            skills_dir: Box::new(|_| home().join(".cursor/rules/obscura-skills.mdc")),
        }),
        "opencode" => Some(ToolConfig {
            name: "OpenCode",
            skills_dir: Box::new(|s| home().join(format!(".opencode/skills/{s}/SKILL.md"))),
        }),
        "cline" => Some(ToolConfig {
            name: "Cline",
            skills_dir: Box::new(|s| home().join(format!(".cline/skills/{s}/SKILL.md"))),
        }),
        _ => None,
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn ensure_parent(path: &std::path::Path) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
}

fn upsert_file(path: &PathBuf, content: &str) -> bool {
    ensure_parent(path);
    if path.exists() {
        println!("  skip (exists): {}", path.display());
        return false;
    }
    match fs::write(path, content) {
        Ok(()) => {
            println!("  wrote: {}", path.display());
            true
        }
        Err(e) => {
            eprintln!("  error writing {}: {e}", path.display());
            false
        }
    }
}

// ── Per-tool transforms ──────────────────────────────────────────────────

pub fn transform_skill(content: &str, tool: &str) -> String {
    match tool {
        "cursor" => content.replacen("---\n", "---\nglobs:\nalwaysApply: true\n", 1),
        _ => content.to_string(),
    }
}

// ── Install / Uninstall ──────────────────────────────────────────────────

pub fn install_tool(tool_id: &str) {
    let cfg = match tool_config(tool_id) {
        Some(c) => c,
        None => {
            eprintln!("Unknown tool: {tool_id}");
            eprintln!(
                "Available: {}",
                ALL_TOOLS
                    .iter()
                    .map(|(id, _)| *id)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            return;
        }
    };

    println!("\nInstalling Obscura for {}...\n", cfg.name);

    println!("[Skills]");
    if tool_id == "cursor" {
        let mut combined = String::from(
            "---\ndescription: Obscura browser automation skills\nglobs:\nalwaysApply: true\n---\n\n# Obscura Browser Skills\n\n",
        );
        for (name, content) in CANONICAL_SKILLS {
            let transformed = transform_skill(content, tool_id);
            combined.push_str(&format!("## {name}\n\n{transformed}\n\n---\n\n"));
        }
        let dest = (cfg.skills_dir)("");
        upsert_file(&dest, &combined);
    } else {
        for (name, content) in CANONICAL_SKILLS {
            let transformed = transform_skill(content, tool_id);
            let dest = (cfg.skills_dir)(name);
            upsert_file(&dest, &transformed);
        }
    }

    println!("\nDone! Obscura installed for {}.\n", cfg.name);
}

pub fn uninstall_tool(tool_id: &str) {
    let cfg = match tool_config(tool_id) {
        Some(c) => c,
        None => {
            eprintln!("Unknown tool: {tool_id}");
            return;
        }
    };

    println!("\nUninstalling Obscura from {}...\n", cfg.name);

    println!("[Skills]");
    if tool_id == "cursor" {
        let dest = (cfg.skills_dir)("");
        if dest.exists() {
            let _ = fs::remove_file(&dest);
            println!("  removed: {}", dest.display());
        }
    } else {
        for (name, _) in CANONICAL_SKILLS {
            let dest = (cfg.skills_dir)(name);
            if dest.exists() {
                let _ = fs::remove_file(&dest);
                println!("  removed: {}", dest.display());
                if let Some(parent) = dest.parent() {
                    let _ = fs::remove_dir(parent);
                }
            }
        }
    }

    println!("\nDone! Obscura uninstalled from {}.\n", cfg.name);
}

// ── List ─────────────────────────────────────────────────────────────────

pub fn list_tools() {
    println!("\nObscura — Install Targets\n");
    println!("  {:<9} {:<15}", "ID", "Tool");
    println!("  {:<9} {:<15}", "─────", "─────");
    for (id, name) in ALL_TOOLS {
        println!("  {id:<9} {name:<15}");
    }
    println!();
}
