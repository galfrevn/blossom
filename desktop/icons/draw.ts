import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { type Drawing, type DrawingName, readDrawing } from "./drawing";
import { encodePng } from "./png";
import { render, renderings } from "./render";

const iconsFolder = import.meta.dir;
const appIconFile = "icon_512x512.png";

const drawings: Record<DrawingName, Drawing> = {
  small: readDrawing(iconsFolder, "small"),
  medium: readDrawing(iconsFolder, "medium"),
  large: readDrawing(iconsFolder, "large"),
};
const scratchFolder = mkdtempSync(join(tmpdir(), "blossom-"));
const iconsetFolder = join(scratchFolder, "icon.iconset");
mkdirSync(iconsetFolder);
for (const rendering of renderings) {
  const png = encodePng(
    rendering.size,
    render(rendering, drawings[rendering.drawing]),
  );
  writeFileSync(join(iconsetFolder, rendering.file), png);
  if (rendering.file === appIconFile) {
    writeFileSync(join(iconsFolder, "icon.png"), png);
  }
}
execFileSync("iconutil", [
  "--convert",
  "icns",
  iconsetFolder,
  "--output",
  join(iconsFolder, "icon.icns"),
]);
if (process.argv.includes("--keep")) {
  process.stdout.write(`${iconsetFolder}\n`);
} else {
  rmSync(scratchFolder, { recursive: true });
}
