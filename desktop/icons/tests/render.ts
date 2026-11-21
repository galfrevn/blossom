import { expect, test } from "bun:test";
import { join } from "node:path";
import {
  type Color,
  type Drawing,
  type DrawingName,
  parseColor,
  readDrawing,
} from "../drawing";
import { type Rendering, render, renderings } from "../render";

const iconsFolder = join(import.meta.dir, "..");
const drawings: Record<DrawingName, Drawing> = {
  small: readDrawing(iconsFolder, "small"),
  medium: readDrawing(iconsFolder, "medium"),
  large: readDrawing(iconsFolder, "large"),
};
const channelsPerPixel = 4;
const opaque = 255;
const transparent = 0;
const smallestMediumMark = 32;
const smallestLargeMark = 80;
const tileColor = parseColor("#120f0c");

function pixelAt(
  pixels: Uint8Array,
  size: number,
  column: number,
  row: number,
): number[] {
  const index = (row * size + column) * channelsPerPixel;
  return [...pixels.subarray(index, index + channelsPerPixel)];
}

function opaquePixel(color: Color): number[] {
  return [color.red, color.green, color.blue, opaque];
}

function renderingOf(size: number): Rendering {
  const rendering = renderings.find((candidate) => candidate.size === size);
  if (rendering === undefined) {
    throw new Error(`no rendering of ${size} px`);
  }
  return rendering;
}

test("the 16 px icon is the 16 grid drawing at its own size, with nothing around it", () => {
  const size = 16;
  const pixels = render(renderingOf(size), drawings.small);
  const painted = new Map(
    drawings.small.cells.map((cell) => [
      `${cell.column},${cell.row}`,
      cell.color,
    ]),
  );

  for (let row = 0; row < size; row += 1) {
    for (let column = 0; column < size; column += 1) {
      const color = painted.get(`${column},${row}`);
      let expected = [transparent, transparent, transparent, transparent];
      if (color !== undefined) {
        expected = opaquePixel(color);
      }
      expect(pixelAt(pixels, size, column, row)).toEqual(expected);
    }
  }
});

test("the 32 px icon sets the 16 grid drawing on the tile, eight pixels in", () => {
  const size = 32;
  const markOffset = 8;
  const pixels = render(
    { file: "icon_32x32.png", size, drawing: "small", scale: 1, onTile: true },
    drawings.small,
  );
  const [firstCell] = drawings.small.cells;
  if (firstCell === undefined) {
    throw new Error("small.svg has no cells");
  }

  expect(pixelAt(pixels, size, 0, 0)[channelsPerPixel - 1]).toBe(transparent);
  expect(pixelAt(pixels, size, size / 2, markOffset / 2)).toEqual(
    opaquePixel(tileColor),
  );
  expect(
    pixelAt(
      pixels,
      size,
      markOffset + firstCell.column,
      markOffset + firstCell.row,
    ),
  ).toEqual(opaquePixel(firstCell.color));
});

test("every size in the icon set renders its drawing on whole pixels", () => {
  for (const rendering of renderings) {
    expect(() => render(rendering, drawings[rendering.drawing])).not.toThrow();
  }
});

test("each size uses the drawing that ADR 0016 gives the size its mark is shown at", () => {
  for (const rendering of renderings) {
    const markSize = drawings[rendering.drawing].grid * rendering.scale;
    let expected: DrawingName = "large";
    if (markSize < smallestMediumMark) {
      expected = "small";
    } else if (markSize < smallestLargeMark) {
      expected = "medium";
    }
    expect(`${rendering.file}: ${rendering.drawing}`).toBe(
      `${rendering.file}: ${expected}`,
    );
  }
});
