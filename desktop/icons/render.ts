import { type Drawing, type DrawingName, parseColor } from "./drawing";

type RoundedSquare = { inset: number; radius: number; size: number };
type Point = { x: number; y: number };
type TileSamples = { borderSamples: number; fillSamples: number };

const referenceSize = 1024;
const tileInset = 100;
const tileRadius = 185.4;
const borderWidth = 4;
const tileColor = parseColor("#120f0c");
const borderColor = parseColor("#2b2722");
const samplesPerAxis = 4;
const halfSample = 0.5;
const opaque = 255;
const channelsPerPixel = 4;

function isInside(point: Point, square: RoundedSquare): boolean {
  const low = square.inset;
  const high = square.size - square.inset;
  if (point.x < low || point.x > high || point.y < low || point.y > high) {
    return false;
  }
  const nearestCenterX = Math.min(
    Math.max(point.x, low + square.radius),
    high - square.radius,
  );
  const nearestCenterY = Math.min(
    Math.max(point.y, low + square.radius),
    high - square.radius,
  );
  return (
    Math.hypot(point.x - nearestCenterX, point.y - nearestCenterY) <=
    square.radius
  );
}

function tileSamplesAt(
  column: number,
  row: number,
  outer: RoundedSquare,
  inner: RoundedSquare,
): TileSamples {
  let borderSamples = 0;
  let fillSamples = 0;
  for (let sampleRow = 0; sampleRow < samplesPerAxis; sampleRow += 1) {
    for (
      let sampleColumn = 0;
      sampleColumn < samplesPerAxis;
      sampleColumn += 1
    ) {
      const point = {
        x: column + (sampleColumn + halfSample) / samplesPerAxis,
        y: row + (sampleRow + halfSample) / samplesPerAxis,
      };
      if (isInside(point, inner)) {
        fillSamples += 1;
      } else if (isInside(point, outer)) {
        borderSamples += 1;
      }
    }
  }
  return { borderSamples, fillSamples };
}

function tilePixel(samples: TileSamples): number[] {
  const covered = samples.borderSamples + samples.fillSamples;
  const blend = (border: number, fill: number): number =>
    Math.round(
      (border * samples.borderSamples + fill * samples.fillSamples) / covered,
    );
  const alpha = Math.round(
    (opaque * covered) / (samplesPerAxis * samplesPerAxis),
  );
  return [
    blend(borderColor.red, tileColor.red),
    blend(borderColor.green, tileColor.green),
    blend(borderColor.blue, tileColor.blue),
    alpha,
  ];
}

function paintTile(pixels: Uint8Array, size: number): void {
  const factor = size / referenceSize;
  const outer = {
    inset: tileInset * factor,
    radius: tileRadius * factor,
    size,
  };
  const inner = {
    inset: (tileInset + borderWidth) * factor,
    radius: (tileRadius - borderWidth) * factor,
    size,
  };
  for (let row = 0; row < size; row += 1) {
    for (let column = 0; column < size; column += 1) {
      const samples = tileSamplesAt(column, row, outer, inner);
      if (samples.borderSamples + samples.fillSamples > 0) {
        pixels.set(
          tilePixel(samples),
          (row * size + column) * channelsPerPixel,
        );
      }
    }
  }
}

function paintMark(
  pixels: Uint8Array,
  size: number,
  drawing: Drawing,
  scale: number,
): void {
  const markOffset = (size - drawing.grid * scale) / 2;
  if (!Number.isInteger(markOffset)) {
    throw new Error(
      `the ${drawing.grid} grid at ${scale}x does not center on whole pixels in ${size} px`,
    );
  }
  for (const cell of drawing.cells) {
    for (let rowWithinCell = 0; rowWithinCell < scale; rowWithinCell += 1) {
      for (
        let columnWithinCell = 0;
        columnWithinCell < scale;
        columnWithinCell += 1
      ) {
        const x = markOffset + cell.column * scale + columnWithinCell;
        const y = markOffset + cell.row * scale + rowWithinCell;
        pixels.set(
          [cell.color.red, cell.color.green, cell.color.blue, opaque],
          (y * size + x) * channelsPerPixel,
        );
      }
    }
  }
}

export type Rendering = {
  file: string;
  size: number;
  drawing: DrawingName;
  scale: number;
  onTile: boolean;
};

export const renderings: Rendering[] = [
  {
    file: "icon_16x16.png",
    size: 16,
    drawing: "small",
    scale: 1,
    onTile: false,
  },
  {
    file: "icon_16x16@2x.png",
    size: 32,
    drawing: "small",
    scale: 1,
    onTile: true,
  },
  {
    file: "icon_32x32.png",
    size: 32,
    drawing: "small",
    scale: 1,
    onTile: true,
  },
  {
    file: "icon_32x32@2x.png",
    size: 64,
    drawing: "medium",
    scale: 1,
    onTile: true,
  },
  {
    file: "icon_128x128.png",
    size: 128,
    drawing: "medium",
    scale: 2,
    onTile: true,
  },
  {
    file: "icon_128x128@2x.png",
    size: 256,
    drawing: "large",
    scale: 3,
    onTile: true,
  },
  {
    file: "icon_256x256.png",
    size: 256,
    drawing: "large",
    scale: 3,
    onTile: true,
  },
  {
    file: "icon_256x256@2x.png",
    size: 512,
    drawing: "large",
    scale: 6,
    onTile: true,
  },
  {
    file: "icon_512x512.png",
    size: 512,
    drawing: "large",
    scale: 6,
    onTile: true,
  },
  {
    file: "icon_512x512@2x.png",
    size: 1024,
    drawing: "large",
    scale: 13,
    onTile: true,
  },
];

export function render(rendering: Rendering, drawing: Drawing): Uint8Array {
  const pixels = new Uint8Array(
    rendering.size * rendering.size * channelsPerPixel,
  );
  if (rendering.onTile) {
    paintTile(pixels, rendering.size);
  }
  paintMark(pixels, rendering.size, drawing, rendering.scale);
  return pixels;
}
