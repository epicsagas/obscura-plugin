#!/usr/bin/env node
// obscura-mcp plugin bootstrap
// Runs on SessionStart via hooks.json.
// Uses only Node.js built-ins — no npm install needed.

"use strict";

const { spawnSync, execSync } = require("child_process");
const { createWriteStream, chmodSync, readFileSync, existsSync, mkdirSync, copyFileSync } = require("fs");
const { join, basename } = require("path");
const https = require("https");
const os = require("os");
const zlib = require("zlib");

const REPO = "epicsagas/obscura-plugin";
const OBSCURA_REPO = "h4ckf0r0day/obscura";
const MCP_BINARY = "obscura-mcp";
const OBSCURA_BINARY = "obscura";

// ── Platform detection ───────────────────────────────────────────────────────

function platform() {
  const p = os.platform();
  const a = os.arch();
  if (p === "darwin") return a === "arm64" ? "aarch64-macos" : "x86_64-macos";
  if (p === "linux")  return "x86_64-linux";
  if (p === "win32")  return "x86_64-windows";
  return null;
}

function assetName(binaryBaseName, plat) {
  return plat === "x86_64-windows"
    ? `${binaryBaseName}-${plat}.zip`
    : `${binaryBaseName}-${plat}.tar.gz`;
}

// ── Install dir + PATH ───────────────────────────────────────────────────────

function installDir() {
  if (os.platform() === "win32") {
    return join(os.homedir(), "AppData", "Local", "Programs", "obscura");
  }
  return join(os.homedir(), ".local", "bin");
}

function ensureInstallDir(dir) {
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
}

function patchShellRc(dir) {
  const exportLine = `export PATH="${dir}:$PATH"`;
  const rcFiles = [".bashrc", ".zshrc", ".profile"].map(f => join(os.homedir(), f));
  for (const rc of rcFiles) {
    if (!existsSync(rc)) continue;
    const content = readFileSync(rc, "utf8");
    if (content.includes(dir)) continue; // already present
    require("fs").appendFileSync(rc, `\n# added by obscura-plugin\n${exportLine}\n`);
    log(`Added ${dir} to ${basename(rc)}`);
  }
}

function resolveExe(name, dir) {
  const inDir = join(dir, os.platform() === "win32" ? `${name}.exe` : name);
  if (existsSync(inDir)) return inDir;
  // also check PATH
  const r = spawnSync(os.platform() === "win32" ? "where" : "which", [name], { stdio: "pipe" });
  if (r.status === 0) return r.stdout.toString().trim().split("\n")[0];
  return null;
}

// ── Logging ──────────────────────────────────────────────────────────────────

function log(msg) {
  process.stderr.write(`[obscura-plugin] ${msg}\n`);
}

// ── Version helpers ──────────────────────────────────────────────────────────

function getBinaryVersion(exePath) {
  try {
    const r = spawnSync(exePath, ["--version"], { stdio: "pipe" });
    if (r.status === 0) {
      const m = r.stdout.toString().match(/(\d+\.\d+\.\d+)/);
      return m ? m[1] : null;
    }
  } catch (_) {}
  return null;
}

function getPluginVersion() {
  try {
    const root = process.env.CLAUDE_PLUGIN_ROOT || __dirname.replace(/[\\/]hooks$/, "");
    const manifest = JSON.parse(readFileSync(join(root, ".claude-plugin", "plugin.json"), "utf8"));
    return manifest.version || null;
  } catch (_) {}
  return null;
}

function semverGt(a, b) {
  const pa = a.split(".").map(Number);
  const pb = b.split(".").map(Number);
  for (let i = 0; i < 3; i++) {
    if (pa[i] > pb[i]) return true;
    if (pa[i] < pb[i]) return false;
  }
  return false;
}

// ── Download ─────────────────────────────────────────────────────────────────

function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    const file = createWriteStream(dest);
    const follow = (u) => {
      https.get(u, { headers: { "User-Agent": "obscura-plugin-installer" } }, (res) => {
        if (res.statusCode === 301 || res.statusCode === 302) {
          res.resume();
          return follow(res.headers.location);
        }
        if (res.statusCode !== 200) {
          return reject(new Error(`HTTP ${res.statusCode} for ${u}`));
        }
        res.pipe(file);
        file.on("finish", () => file.close(resolve));
      }).on("error", reject);
    };
    follow(url);
  });
}

// ── Extract ──────────────────────────────────────────────────────────────────

async function extractBinary(archive, binaryName, destDir) {
  if (archive.endsWith(".zip")) {
    // Windows: use PowerShell to expand zip
    const ps = `Expand-Archive -Path '${archive}' -DestinationPath '${destDir}' -Force`;
    const r = spawnSync("powershell", ["-Command", ps], { stdio: "inherit" });
    if (r.status !== 0) throw new Error("Failed to extract zip");
    return join(destDir, `${binaryName}.exe`);
  }

  // tar.gz: use Node streams (no external tar needed)
  const { createReadStream } = require("fs");
  const extracted = await new Promise((resolve, reject) => {
    const tar = require("child_process").spawn("tar", ["-xz", "-C", destDir, "--strip-components=0"], {
      stdio: ["pipe", "inherit", "inherit"],
    });
    createReadStream(archive).pipe(tar.stdin);
    tar.on("close", (code) => {
      if (code === 0) resolve(join(destDir, binaryName));
      else reject(new Error(`tar exited with ${code}`));
    });
  });
  return extracted;
}

// ── GitHub release asset URL ─────────────────────────────────────────────────

async function getLatestAssetUrl(repo, assetFilename) {
  return new Promise((resolve, reject) => {
    const url = `https://api.github.com/repos/${repo}/releases/latest`;
    https.get(url, { headers: { "User-Agent": "obscura-plugin-installer", "Accept": "application/vnd.github+json" } }, (res) => {
      let data = "";
      res.on("data", (c) => (data += c));
      res.on("end", () => {
        try {
          const rel = JSON.parse(data);
          const asset = (rel.assets || []).find((a) => a.name === assetFilename);
          if (!asset) return reject(new Error(`Asset ${assetFilename} not found in ${repo} latest release`));
          resolve(asset.browser_download_url);
        } catch (e) {
          reject(e);
        }
      });
    }).on("error", reject);
  });
}

// ── Install binary from GitHub release ──────────────────────────────────────

async function installFromRelease(repo, binaryBaseName, binaryName, destDir) {
  const plat = platform();
  if (!plat) {
    log(`Unsupported platform: ${os.platform()}/${os.arch()}`);
    return null;
  }

  const filename = assetName(binaryBaseName, plat);
  log(`Fetching ${binaryName} release asset: ${filename}`);

  let assetUrl;
  try {
    assetUrl = await getLatestAssetUrl(repo, filename);
  } catch (e) {
    throw new Error(`Could not find release asset: ${e.message}`);
  }

  const tmp = join(os.tmpdir(), filename);
  log(`Downloading ${assetUrl}`);
  await downloadFile(assetUrl, tmp);

  ensureInstallDir(destDir);
  const exeName = plat === "x86_64-windows" ? `${binaryName}.exe` : binaryName;
  const dest = join(destDir, exeName);

  const extracted = await extractBinary(tmp, exeName, os.tmpdir());
  copyFileSync(extracted, dest);
  if (plat !== "x86_64-windows") chmodSync(dest, 0o755);

  log(`Installed ${binaryName} → ${dest}`);
  return dest;
}

// ── Seed (MCP register + skills) ─────────────────────────────────────────────

function seed(mcpExe) {
  const r = spawnSync(mcpExe, ["install", "claude"], { stdio: "inherit" });
  if (r.status !== 0) log(`Warning: 'obscura-mcp install claude' exited ${r.status}`);
}

// ── Main ─────────────────────────────────────────────────────────────────────

async function main() {
  const dir = installDir();
  const pluginVersion = getPluginVersion();

  // ── 1. Ensure obscura ───────────────────────────────────────────────────
  let obscuraExe = resolveExe(OBSCURA_BINARY, dir);
  if (!obscuraExe) {
    log(`${OBSCURA_BINARY} not found — installing...`);
    try {
      obscuraExe = await installFromRelease(OBSCURA_REPO, "obscura", OBSCURA_BINARY, dir);
      patchShellRc(dir);
      // verify
      if (!obscuraExe || !getBinaryVersion(obscuraExe)) {
        log(`${OBSCURA_BINARY} installed but could not verify. Set OBSCURA_BIN=${join(dir, OBSCURA_BINARY)} if needed.`);
      } else {
        log(`${OBSCURA_BINARY} ${getBinaryVersion(obscuraExe)} ready`);
      }
    } catch (e) {
      log(`${OBSCURA_BINARY} install failed: ${e.message}`);
      log(`Install manually: https://github.com/${OBSCURA_REPO}#installation`);
    }
  }

  // ── 2. Ensure obscura-mcp ───────────────────────────────────────────────
  let mcpExe = resolveExe(MCP_BINARY, dir);
  if (!mcpExe) {
    log(`${MCP_BINARY} not found — installing...`);
    try {
      mcpExe = await installFromRelease(REPO, "obscura-mcp", MCP_BINARY, dir);
      patchShellRc(dir);
      if (mcpExe && getBinaryVersion(mcpExe)) {
        log(`${MCP_BINARY} ${getBinaryVersion(mcpExe)} ready`);
        seed(mcpExe);
      }
    } catch (e) {
      log(`${MCP_BINARY} install failed: ${e.message}`);
      log(`Install manually: https://github.com/${REPO}/releases`);
      process.exit(0);
    }
    return;
  }

  // ── 3. Update obscura-mcp if plugin version is newer ────────────────────
  if (pluginVersion) {
    const binaryVersion = getBinaryVersion(mcpExe);
    if (binaryVersion && semverGt(pluginVersion, binaryVersion)) {
      log(`Updating ${MCP_BINARY} ${binaryVersion} → ${pluginVersion}...`);
      try {
        const updated = await installFromRelease(REPO, "obscura-mcp", MCP_BINARY, dir);
        if (updated) {
          mcpExe = updated;
          log(`Updated to ${getBinaryVersion(mcpExe)}`);
        }
      } catch (e) {
        log(`Update failed: ${e.message} — continuing with ${binaryVersion}`);
      }
    }
  }

  // ── 4. Seed MCP config + skills ─────────────────────────────────────────
  seed(mcpExe);
}

main().catch((e) => {
  log(`Unexpected error: ${e.message}`);
  process.exit(0);
});
