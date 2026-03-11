/**
 * ensure-ollama.mjs
 *
 * Downloads the Ollama sidecar binary into src-tauri/binaries/ if it is not
 * already present.  Runs automatically before every `npm run tauri:build` via
 * the "pretauri:build" npm lifecycle hook — no manual steps needed.
 *
 * Works on Windows (x86_64) and macOS (arm64 + x86_64 for universal builds).
 */

import https       from "node:https";
import fs          from "node:fs";
import path        from "node:path";
import os          from "node:os";
import { execSync } from "node:child_process";
import { createWriteStream } from "node:fs";
import { pipeline } from "node:stream/promises";
import { createGunzip } from "node:zlib";

const ROOT    = path.resolve(import.meta.dirname, "..");
const BIN_DIR = path.join(ROOT, "src-tauri", "binaries");

// ── Target triple → expected filename in BIN_DIR ─────────────────────────────
const TARGETS = {
  "win32-x64":   { dest: "ollama-x86_64-pc-windows-msvc.exe",  ext: "exe" },
  "darwin-arm64":{ dest: "ollama-aarch64-apple-darwin",         ext: "bin" },
  "darwin-x64":  { dest: "ollama-x86_64-apple-darwin",          ext: "bin" },
};

// ── Pick targets based on current platform ────────────────────────────────────
function neededTargets() {
  const p = `${process.platform}-${os.arch()}`;
  if (process.platform === "darwin") {
    // Always download both archs so a universal build works.
    return [TARGETS["darwin-arm64"], TARGETS["darwin-x64"]];
  }
  if (TARGETS[p]) return [TARGETS[p]];
  console.error(`[ensure-ollama] Unsupported platform: ${p}`);
  process.exit(1);
}

// ── HTTPS fetch with redirect following ───────────────────────────────────────
function httpsGet(url) {
  return new Promise((resolve, reject) => {
    https.get(url, { headers: { "User-Agent": "MeridianBuild/1.0" } }, (res) => {
      if (res.statusCode === 301 || res.statusCode === 302) {
        return httpsGet(res.headers.location).then(resolve).catch(reject);
      }
      resolve(res);
    }).on("error", reject);
  });
}

async function fetchJson(url) {
  const res = await httpsGet(url);
  return new Promise((resolve, reject) => {
    let body = "";
    res.on("data", (c) => (body += c));
    res.on("end", () => {
      try { resolve(JSON.parse(body)); }
      catch (e) { reject(new Error(`JSON parse error from ${url}: ${e.message}`)); }
    });
    res.on("error", reject);
  });
}

// ── Download file with progress bar ───────────────────────────────────────────
async function downloadFile(url, dest) {
  const res      = await httpsGet(url);
  const total    = parseInt(res.headers["content-length"] || "0", 10);
  let   received = 0;

  const tmpFile = dest + ".tmp";
  const out     = createWriteStream(tmpFile);

  res.on("data", (chunk) => {
    received += chunk.length;
    if (total > 0) {
      const pct  = Math.round((received / total) * 100);
      const mb   = (received / 1024 / 1024).toFixed(1);
      const tot  = (total  / 1024 / 1024).toFixed(1);
      process.stdout.write(`\r  ${pct}%  ${mb} / ${tot} MB`);
    }
  });

  await pipeline(res, out);
  process.stdout.write("\n");
  fs.renameSync(tmpFile, dest);
}

// ── Extract a single binary from a tar.gz (macOS) ────────────────────────────
async function extractFromTarGz(archive, outFile) {
  // Use system tar — always present on macOS/Linux
  const dir = path.dirname(outFile);
  execSync(`tar -xzf "${archive}" -C "${dir}" ollama`, { stdio: "inherit" });
  const extracted = path.join(dir, "ollama");
  if (fs.existsSync(extracted)) {
    fs.renameSync(extracted, outFile);
  } else {
    throw new Error(`'ollama' not found inside ${archive}`);
  }
}

// ── Extract a .zip (Windows or macOS zip) ────────────────────────────────────
function extractFromZip(archive, outFile) {
  // Use a unique temp subdirectory to avoid collisions with other files.
  const extractDir = archive + "_extracted";
  fs.mkdirSync(extractDir, { recursive: true });

  if (process.platform === "win32") {
    execSync(
      `powershell -NoProfile -Command "Expand-Archive -LiteralPath '${archive}' -DestinationPath '${extractDir}' -Force"`,
      { stdio: "inherit" }
    );
    // Recursively search for ollama.exe anywhere inside the extracted tree.
    const found = findFileRecursive(extractDir, "ollama.exe");
    if (!found) {
      // Show what was actually extracted to help debug future asset changes.
      const contents = listFilesRecursive(extractDir).slice(0, 20).join("\n  ");
      throw new Error(
        `ollama.exe not found inside the ZIP.\nExtracted files:\n  ${contents}`
      );
    }
    fs.copyFileSync(found, outFile);
  } else {
    execSync(`unzip -o "${archive}" -d "${extractDir}"`, { stdio: "inherit" });
    const found = findFileRecursive(extractDir, "ollama");
    if (!found) throw new Error(`'ollama' binary not found inside ${archive}`);
    fs.copyFileSync(found, outFile);
  }

  fs.rmSync(extractDir, { recursive: true, force: true });
}

function findFileRecursive(dir, name) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      const found = findFileRecursive(full, name);
      if (found) return found;
    } else if (entry.name.toLowerCase() === name.toLowerCase()) {
      return full;
    }
  }
  return null;
}

function listFilesRecursive(dir, results = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) listFilesRecursive(full, results);
    else results.push(full);
  }
  return results;
}

// ── Check if a locally installed Ollama binary can be reused ─────────────────
function findLocalOllama() {
  const candidates = [];

  if (process.platform === "win32") {
    candidates.push(
      path.join(process.env.LOCALAPPDATA || "", "Programs", "Ollama", "ollama.exe"),
      path.join(process.env.PROGRAMFILES || "", "Ollama", "ollama.exe"),
    );
  } else {
    candidates.push(
      "/usr/local/bin/ollama",
      path.join(os.homedir(), ".ollama", "ollama"),
    );
  }

  // Also search PATH
  try {
    const which = execSync(
      process.platform === "win32" ? "where ollama" : "which ollama",
      { stdio: ["ignore", "pipe", "ignore"] }
    ).toString().trim().split(/\r?\n/)[0];
    if (which) candidates.unshift(which);
  } catch { /* not in PATH */ }

  return candidates.find((p) => p && fs.existsSync(p)) || null;
}

// ── Create macOS universal binary from the two arch-specific ones ─────────────
// Recent Ollama releases ship a single fat (arm64+x86_64) universal binary for
// macOS.  Both arch targets download the same file, so lipo -create would fail.
// Detect that case and just copy the file instead of combining.
function createUniversalBinary(binDir) {
  const arm64 = path.join(binDir, "ollama-aarch64-apple-darwin");
  const x64   = path.join(binDir, "ollama-x86_64-apple-darwin");
  const univ  = path.join(binDir, "ollama-universal-apple-darwin");

  if (!fs.existsSync(arm64) || !fs.existsSync(x64)) {
    console.warn("[ensure-ollama] Skipping universal binary: arch-specific binaries not both present");
    return;
  }

  // Check if the downloaded binary is already a fat (universal) Mach-O.
  let lipoInfo = "";
  try { lipoInfo = execSync(`lipo -info "${arm64}" 2>&1`, { encoding: "utf8" }); } catch { /* not on macOS or lipo unavailable */ }
  const alreadyUniversal = lipoInfo.includes("arm64") && lipoInfo.includes("x86_64");

  if (alreadyUniversal) {
    // Ollama ships a pre-built universal binary — just copy it directly.
    console.log("[ensure-ollama] Ollama binary is already universal — copying directly");
    fs.copyFileSync(arm64, univ);
  } else {
    // Separate thin binaries — combine them into a fat binary.
    console.log("[ensure-ollama] Creating universal binary via lipo -create...");
    execSync(`lipo -create -output "${univ}" "${arm64}" "${x64}"`, { stdio: "inherit" });
  }
  fs.chmodSync(univ, 0o755);
  const sizeMB = (fs.statSync(univ).size / 1024 / 1024).toFixed(1);
  console.log(`[ensure-ollama] OK  ollama-universal-apple-darwin  (${sizeMB} MB)`);
}

// ── Main ──────────────────────────────────────────────────────────────────────
async function main() {
  fs.mkdirSync(BIN_DIR, { recursive: true });

  const targets = neededTargets();
  const missing = targets.filter((t) => !fs.existsSync(path.join(BIN_DIR, t.dest)));

  if (missing.length === 0) {
    for (const t of targets) {
      const sizeMB = (fs.statSync(path.join(BIN_DIR, t.dest)).size / 1024 / 1024).toFixed(1);
      console.log(`[ensure-ollama] OK  ${t.dest}  (${sizeMB} MB) — already present`);
    }
    if (process.platform === "darwin") createUniversalBinary(BIN_DIR);
    return;
  }

  // ── Try to reuse a locally installed binary (developer convenience) ─────────
  const localBin = findLocalOllama();
  if (localBin && missing.length === 1) {
    const dest    = path.join(BIN_DIR, missing[0].dest);
    const sizeMB  = (fs.statSync(localBin).size / 1024 / 1024).toFixed(1);
    console.log(`[ensure-ollama] Copying local Ollama (${sizeMB} MB): ${localBin}`);
    fs.copyFileSync(localBin, dest);
    if (process.platform !== "win32") fs.chmodSync(dest, 0o755);
    console.log(`[ensure-ollama] OK  ${missing[0].dest}`);
    return;
  }

  // ── Fetch latest GitHub release ─────────────────────────────────────────────
  console.log("[ensure-ollama] Fetching Ollama release info from GitHub...");
  const release = await fetchJson(
    "https://api.github.com/repos/ollama/ollama/releases/latest"
  );
  const version = release.tag_name;
  console.log(`[ensure-ollama] Latest Ollama: ${version}`);

  // ── Download each missing binary ────────────────────────────────────────────
  for (const target of missing) {
    const dest = path.join(BIN_DIR, target.dest);

    // Find the best matching asset for this target.
    // GPU-specific builds (rocm, cuda) are skipped — they have different
    // internal structures and are much larger than the standard binary.
    const isGpuBuild = (n) => n.includes("rocm") || n.includes("cuda");

    let assetUrl  = null;
    let assetName = null;

    if (target.dest.startsWith("ollama-x86_64-pc-windows")) {
      // Priority 1: plain ollama-windows-amd64.zip (no GPU suffix)
      const plainZip = release.assets.find(
        (a) => a.name.toLowerCase() === "ollama-windows-amd64.zip"
      );
      // Priority 2: any windows amd64 zip that isn't GPU-specific or an installer
      const anyZip = release.assets.find((a) => {
        const n = a.name.toLowerCase();
        return n.includes("windows") && n.includes("amd64") &&
               n.endsWith(".zip") && !isGpuBuild(n);
      });
      // Priority 3: OllamaSetup.exe
      const setup = release.assets.find(
        (a) => a.name.toLowerCase() === "ollamasetup.exe"
      );
      const chosen = plainZip || anyZip || setup;
      if (chosen) { assetUrl = chosen.browser_download_url; assetName = chosen.name; }
    } else {
      for (const asset of release.assets) {
        const n = asset.name.toLowerCase();
        const isArm64 = n.includes("darwin") && (n.includes("arm64") || n.includes("aarch64"));
        const isAmd64 = n.includes("darwin") && n.includes("amd64");
        if (target.dest.includes("aarch64-apple") && isArm64) {
          assetUrl = asset.browser_download_url; assetName = asset.name; break;
        }
        if (target.dest.includes("x86_64-apple") && isAmd64) {
          assetUrl = asset.browser_download_url; assetName = asset.name; break;
        }
      }
    }

    // macOS: fall back to the combined darwin zip/tgz if per-arch not found
    if (!assetUrl && process.platform === "darwin") {
      const fallback = release.assets.find((a) => {
        const n = a.name.toLowerCase();
        return n.includes("darwin") && !isGpuBuild(n) &&
               (n.endsWith(".zip") || n.endsWith(".tgz") || n.endsWith(".tar.gz"));
      });
      if (fallback) {
        assetUrl  = fallback.browser_download_url;
        assetName = fallback.name;
      }
    }

    if (!assetUrl) {
      console.error(`\n[ensure-ollama] Could not find a download for ${target.dest}`);
      console.error("  Available assets:");
      release.assets.forEach((a) => console.error(`    - ${a.name}`));
      process.exit(1);
    }

    const tmpPath = path.join(os.tmpdir(), assetName);
    console.log(`\n[ensure-ollama] Downloading ${assetName}...`);
    await downloadFile(assetUrl, tmpPath);

    const lowerName = assetName.toLowerCase();
    if (lowerName.endsWith(".zip")) {
      console.log("  Extracting from ZIP...");
      extractFromZip(tmpPath, dest);
    } else if (lowerName.endsWith(".tgz") || lowerName.endsWith(".tar.gz")) {
      console.log("  Extracting from tar.gz...");
      await extractFromTarGz(tmpPath, dest);
    } else {
      // Plain binary (.exe or no extension)
      fs.renameSync(tmpPath, dest);
    }

    if (fs.existsSync(tmpPath)) fs.unlinkSync(tmpPath);
    if (process.platform !== "win32") fs.chmodSync(dest, 0o755);

    const sizeMB = (fs.statSync(dest).size / 1024 / 1024).toFixed(1);
    console.log(`[ensure-ollama] OK  ${target.dest}  (${sizeMB} MB)`);
  }

  if (process.platform === "darwin") createUniversalBinary(BIN_DIR);

  console.log("\n[ensure-ollama] All Ollama binaries ready. Proceeding with Tauri build...\n");
}

main().catch((err) => {
  console.error(`\n[ensure-ollama] FAILED: ${err.message}`);
  process.exit(1);
});
