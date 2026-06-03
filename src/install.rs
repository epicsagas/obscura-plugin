use std::fs;
use std::path::PathBuf;

// ── Canonical sources ─────────────────────────────────────────────────────

static SKILL_FETCH: &str = include_str!("../skills/obscura-fetch/SKILL.md");
static SKILL_SCRAPE: &str = include_str!("../skills/obscura-scrape/SKILL.md");
static SKILL_PIPELINE: &str = include_str!("../skills/obscura-pipeline/SKILL.md");
static SKILL_CRAWL: &str = include_str!("../skills/obscura-crawl/SKILL.md");

static CANONICAL_SKILLS: &[(&str, &str)] = &[
    ("obscura-fetch", SKILL_FETCH),
    ("obscura-scrape", SKILL_SCRAPE),
    ("obscura-pipeline", SKILL_PIPELINE),
    ("obscura-crawl", SKILL_CRAWL),
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

// ── Obscura binary install ───────────────────────────────────────────────

const OBSCURA_REPO: &str = "h4ckf0r0day/obscura";

fn obscura_platform() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => Some("aarch64-macos"),
        ("macos", "x86_64") => Some("x86_64-macos"),
        ("linux", "x86_64") => Some("x86_64-linux"),
        ("linux", "aarch64") => Some("aarch64-linux"),
        ("windows", "x86_64") => Some("x86_64-windows"),
        _ => None,
    }
}

fn binary_dir() -> PathBuf {
    #[cfg(windows)]
    {
        home()
            .join("AppData")
            .join("Local")
            .join("Programs")
            .join("obscura")
    }
    #[cfg(not(windows))]
    {
        home().join(".local").join("bin")
    }
}

fn run_cmd(cmd: &str, args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("{cmd}: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{cmd} failed: {}", stderr.trim()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn is_obscura_on_path() -> bool {
    let cmd = if cfg!(windows) { "where" } else { "which" };
    run_cmd(cmd, &["obscura"]).is_ok()
}

fn query_latest_release(repo: &str) -> Result<serde_json::Value, String> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let json = run_cmd(
        "curl",
        &["-sL", "-H", "Accept: application/vnd.github+json", &url],
    )?;
    serde_json::from_str(&json).map_err(|e| format!("parse: {e}"))
}

fn find_asset_url(release: &serde_json::Value, filename: &str) -> Option<String> {
    release["assets"].as_array().and_then(|assets| {
        assets
            .iter()
            .find(|a| a["name"].as_str() == Some(filename))
            .and_then(|a| a["browser_download_url"].as_str().map(String::from))
    })
}

fn patch_shell_rc(dir: &std::path::Path) {
    #[cfg(unix)]
    {
        let export_line = format!("export PATH=\"{}:$PATH\"", dir.display());
        for rc in [".bashrc", ".zshrc", ".profile"] {
            let rc_path = home().join(rc);
            if !rc_path.exists() {
                continue;
            }
            let content = fs::read_to_string(&rc_path).unwrap_or_default();
            if content.contains(dir.to_str().unwrap_or("")) {
                continue;
            }
            if let Ok(mut f) = fs::OpenOptions::new().append(true).open(&rc_path) {
                use std::io::Write;
                let _ = writeln!(f, "\n# added by obscura-plugin\n{export_line}");
                println!("  added {} to ~/{rc}", dir.display());
            }
        }
    }
    let _ = dir; // suppress unused on non-unix
}

/// Download and install `obscura` (+ `obscura-worker`) from GitHub releases.
/// No-op if already on PATH or in `binary_dir()`.
pub fn ensure_obscura_binary() {
    let dir = binary_dir();
    let bin_name = if cfg!(windows) {
        "obscura.exe"
    } else {
        "obscura"
    };
    let dest = dir.join(bin_name);

    // Already installed (in install dir or on PATH)?
    if dest.exists() || is_obscura_on_path() {
        return;
    }

    let plat = match obscura_platform() {
        Some(p) => p,
        None => {
            eprintln!(
                "  Unsupported platform for obscura binary ({} {})",
                std::env::consts::OS,
                std::env::consts::ARCH
            );
            return;
        }
    };

    let ext = if cfg!(windows) { ".zip" } else { ".tar.gz" };
    let asset_name = format!("obscura-{plat}{ext}");

    println!("[Obscura Binary]");
    println!("  Downloading obscura for {plat}...");

    // Query GitHub API
    let release = match query_latest_release(OBSCURA_REPO) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("  Failed to query GitHub: {e}");
            eprintln!("  Install manually: https://github.com/{OBSCURA_REPO}/releases");
            return;
        }
    };

    let download_url = match find_asset_url(&release, &asset_name) {
        Some(u) => u,
        None => {
            eprintln!("  Asset {asset_name} not found in latest release");
            return;
        }
    };

    // Download
    let tmp = std::env::temp_dir().join(&asset_name);
    if let Err(e) = run_cmd(
        "curl",
        &["-sL", "-o", tmp.to_str().unwrap_or(""), &download_url],
    ) {
        eprintln!("  Download failed: {e}");
        return;
    }

    // Extract
    let extract_dir = std::env::temp_dir().join("obscura-extract");
    let _ = fs::remove_dir_all(&extract_dir);
    let _ = fs::create_dir_all(&extract_dir);

    #[cfg(unix)]
    let extract_result = run_cmd(
        "tar",
        &[
            "-xzf",
            tmp.to_str().unwrap_or(""),
            "-C",
            extract_dir.to_str().unwrap_or(""),
        ],
    );

    #[cfg(windows)]
    let extract_result = run_cmd(
        "powershell",
        &[
            "-Command",
            &format!(
                "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                tmp.display(),
                extract_dir.display()
            ),
        ],
    );

    if let Err(e) = extract_result {
        eprintln!("  Extraction failed: {e}");
        let _ = fs::remove_dir_all(&extract_dir);
        let _ = fs::remove_file(&tmp);
        return;
    }

    // Ensure install dir
    let _ = fs::create_dir_all(&dir);

    // Copy obscura
    let src_bin = extract_dir.join(bin_name);
    if src_bin.exists() {
        if let Err(e) = fs::copy(&src_bin, &dest) {
            eprintln!("  Copy failed: {e}");
        } else {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&dest, fs::Permissions::from_mode(0o755));
            }
            println!("  installed: {}", dest.display());
        }
    } else {
        eprintln!("  Binary not found in archive");
    }

    // Copy obscura-worker
    let worker_name = if cfg!(windows) {
        "obscura-worker.exe"
    } else {
        "obscura-worker"
    };
    let src_worker = extract_dir.join(worker_name);
    if src_worker.exists() {
        let worker_dest = dir.join(worker_name);
        if let Err(e) = fs::copy(&src_worker, &worker_dest) {
            eprintln!("  Copy worker failed: {e}");
        } else {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&worker_dest, fs::Permissions::from_mode(0o755));
            }
            println!("  installed: {}", worker_dest.display());
        }
    }

    // Patch shell RC
    patch_shell_rc(&dir);

    // Cleanup
    let _ = fs::remove_dir_all(&extract_dir);
    let _ = fs::remove_file(&tmp);
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

    // Ensure obscura binary is available
    ensure_obscura_binary();

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
