/**
 * Meridian Icon Generator
 * Generates all required icon sizes from a single SVG source.
 * Run: node scripts/generate-icons.mjs
 * Requires: npm install sharp (globally or locally)
 */
import { createCanvas } from "canvas";
import { writeFileSync, mkdirSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ICONS_DIR = join(__dirname, "../src-tauri/icons");
mkdirSync(ICONS_DIR, { recursive: true });

/**
 * Draws the Meridian "M" logo onto a canvas.
 */
function drawIcon(size, dark = true) {
  const canvas  = createCanvas(size, size);
  const ctx     = canvas.getContext("2d");
  const radius  = size * 0.22;
  const cx      = size / 2;
  const cy      = size / 2;

  // Background gradient
  const grad = ctx.createLinearGradient(0, 0, size, size);
  grad.addColorStop(0, "#4f46e5"); // indigo-600
  grad.addColorStop(1, "#7c3aed"); // violet-600

  // Rounded square background
  ctx.beginPath();
  ctx.moveTo(cx + size * 0.5 - radius, cy - size * 0.5);
  ctx.arcTo(cx + size * 0.5, cy - size * 0.5, cx + size * 0.5, cy - size * 0.5 + radius, radius);
  ctx.arcTo(cx + size * 0.5, cy + size * 0.5, cx + size * 0.5 - radius, cy + size * 0.5, radius);
  ctx.arcTo(cx - size * 0.5, cy + size * 0.5, cx - size * 0.5, cy + size * 0.5 - radius, radius);
  ctx.arcTo(cx - size * 0.5, cy - size * 0.5, cx + size * 0.5 - radius, cy - size * 0.5, radius);
  ctx.closePath();
  ctx.fillStyle = grad;
  ctx.fill();

  // Letter M
  ctx.fillStyle = "#ffffff";
  ctx.font      = `bold ${Math.round(size * 0.55)}px Arial, sans-serif`;
  ctx.textAlign    = "center";
  ctx.textBaseline = "middle";
  ctx.fillText("M", cx, cy);

  return canvas;
}

// Generate PNG icons
const SIZES = [16, 32, 64, 128, 256, 512];
for (const size of SIZES) {
  const canvas = drawIcon(size);
  const buf    = canvas.toBuffer("image/png");
  writeFileSync(join(ICONS_DIR, `${size}x${size}.png`), buf);
  console.log(`✓ Generated ${size}x${size}.png`);
}

// Also write 128x128@2x
{
  const canvas = drawIcon(256);
  writeFileSync(join(ICONS_DIR, "128x128@2x.png"), canvas.toBuffer("image/png"));
  console.log("✓ Generated 128x128@2x.png");
}

// Tray icon (small)
{
  const canvas = drawIcon(32);
  writeFileSync(join(ICONS_DIR, "tray-icon.png"), canvas.toBuffer("image/png"));
  console.log("✓ Generated tray-icon.png");
}

console.log("\nNote: .ico and .icns files need to be generated separately.");
console.log("On macOS: png2icns icon.icns icon.png");
console.log("On Windows: Use ImageMagick: magick convert icon.png icon.ico");
console.log("\nFor production builds, use: npx @tauri-apps/cli icon src-tauri/icons/128x128.png");
