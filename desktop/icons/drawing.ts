import { readFileSync } from "node:fs";
import { join } from "node:path";

const hexChannelLength = 2;
const hexadecimal = 16;
const decimal = 10;
const viewBoxPattern = /viewBox="0 0 (?<grid>\d+) \d+"/u;
const runPattern =
  /<rect x="(?<x>\d+)" y="(?<y>\d+)" width="(?<width>\d+)" height="1" fill="(?<fill>#[0-9a-f]{6})"\/>/gu;

export type Color = { red: number; green: number; blue: number };
export type Cell = { column: number; row: number; color: Color };
export type Drawing = { grid: number; cells: Cell[] };
export type DrawingName = "small" | "medium" | "large";

export function parseColor(hex: string): Color {
  const channel = (start: number): number =>
    Number.parseInt(hex.slice(start, start + hexChannelLength), hexadecimal);
  return {
    red: channel(1),
    green: channel(1 + hexChannelLength),
    blue: channel(1 + hexChannelLength * 2),
  };
}

export function readDrawing(folder: string, name: DrawingName): Drawing {
  const svg = readFileSync(join(folder, `${name}.svg`), "utf8");
  const grid = svg.match(viewBoxPattern)?.groups?.grid;
  if (grid === undefined) {
    throw new Error(`${name}.svg has no square viewBox`);
  }
  const cells: Cell[] = [];
  for (const match of svg.matchAll(runPattern)) {
    const { x, y, width, fill } = match.groups ?? {};
    const color = parseColor(String(fill));
    for (let offset = 0; offset < Number(width); offset += 1) {
      cells.push({ column: Number(x) + offset, row: Number(y), color });
    }
  }
  return { grid: Number.parseInt(grid, decimal), cells };
}
