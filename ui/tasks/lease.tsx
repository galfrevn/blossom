import { type Clock, useReadingOf } from "../core/clock";
import type { Lease } from "../core/contract";
import { dictionary } from "../dictionary";

const MILLISECONDS_IN_A_SECOND = 1000;
const LEASE_BOARD_GIVES_WHEN_NONE_IS_ASKED_IN_SECONDS = 900;
const SECONDS_LEFT_WHEN_A_LEASE_RUNS_LOW = 60;
const TRACK_WIDTH = 54;
const TRACK_HEIGHT = 6;
const HAIRLINE = 1;
const HALF_A_HAIRLINE = HAIRLINE / 2;

const STEPS_TO_A_BAYER_GRID_OF_EIGHT = 3;
const QUADRANTS = 4;
const TOP_LEFT = 0;
const TOP_RIGHT = 2;
const BOTTOM_LEFT = 3;
const BOTTOM_RIGHT = 1;
const ORDER_OF_THE_QUADRANTS = [
  [TOP_LEFT, TOP_RIGHT],
  [BOTTOM_LEFT, BOTTOM_RIGHT],
];
const MIDDLE_OF_A_LEVEL = 0.5;
const PAPER_SHOWING_THROUGH_A_LEASE = 0.28;
const DITHER_PIXEL_SIZE = 2;
const LEASE_DITHER = "lease-dither";

function biggerBayerGrid(grid: number[][]): number[][] {
  const size = grid.length;
  const bigger: number[][] = [];
  for (let row = 0; row < size * 2; row += 1) {
    const quadrantRow = ORDER_OF_THE_QUADRANTS[Math.floor(row / size)] ?? [];
    const innerRow = grid[row % size] ?? [];
    const cells: number[] = [];
    for (let column = 0; column < size * 2; column += 1) {
      const quadrant = quadrantRow[Math.floor(column / size)] ?? 0;
      const inner = innerRow[column % size] ?? 0;
      cells.push(inner * QUADRANTS + quadrant);
    }
    bigger.push(cells);
  }
  return bigger;
}

function bayerGrid(steps: number): number[][] {
  let grid = [[0]];
  for (let step = 0; step < steps; step += 1) {
    grid = biggerBayerGrid(grid);
  }
  return grid;
}

const BAYER_GRID = bayerGrid(STEPS_TO_A_BAYER_GRID_OF_EIGHT);
const LEVELS_OF_THE_BAYER_GRID = BAYER_GRID.length * BAYER_GRID.length;
const DITHER_TILE_SIZE = BAYER_GRID.length * DITHER_PIXEL_SIZE;

function pixelsOfPaperShowingThrough(): string {
  const pixels: string[] = [];
  for (const [row, levels] of BAYER_GRID.entries()) {
    for (const [column, level] of levels.entries()) {
      const threshold = (level + MIDDLE_OF_A_LEVEL) / LEVELS_OF_THE_BAYER_GRID;
      if (PAPER_SHOWING_THROUGH_A_LEASE > threshold) {
        const x = column * DITHER_PIXEL_SIZE;
        const y = row * DITHER_PIXEL_SIZE;
        pixels.push(
          `M${x} ${y}h${DITHER_PIXEL_SIZE}v${DITHER_PIXEL_SIZE}h-${DITHER_PIXEL_SIZE}z`,
        );
      }
    }
  }
  return pixels.join("");
}

const PAPER_SHOWING_THROUGH = pixelsOfPaperShowingThrough();

function secondsLeftOf(lease: Lease, now: number): number {
  const millisecondsLeft = Date.parse(lease.until) - now;
  return Math.max(0, Math.ceil(millisecondsLeft / MILLISECONDS_IN_A_SECOND));
}

function lengthInSecondsOf(lease: Lease): number {
  if (lease.since === null) {
    return LEASE_BOARD_GIVES_WHEN_NONE_IS_ASKED_IN_SECONDS;
  }
  const milliseconds = Date.parse(lease.until) - Date.parse(lease.since);
  return Math.max(1, Math.round(milliseconds / MILLISECONDS_IN_A_SECOND));
}

function fillOf(secondsLeft: number): string {
  if (secondsLeft < SECONDS_LEFT_WHEN_A_LEASE_RUNS_LOW) {
    return "fill-state-blocked";
  }
  return "fill-state-claimed";
}

export function LeaseDither() {
  return (
    <svg aria-hidden={true} className="absolute size-0">
      <defs>
        <pattern
          id={LEASE_DITHER}
          width={DITHER_TILE_SIZE}
          height={DITHER_TILE_SIZE}
          patternUnits="userSpaceOnUse"
        >
          <path className="fill-paper" d={PAPER_SHOWING_THROUGH} />
        </pattern>
      </defs>
    </svg>
  );
}

export function LeaseLeft({ lease, clock }: { lease: Lease; clock: Clock }) {
  const secondsLeft = useReadingOf(clock, (now) => secondsLeftOf(lease, now));
  if (secondsLeft === 0) {
    return (
      <span className="flex-none font-mono text-[11px] text-state-blocked">
        {dictionary.tasks.lapsed}
      </span>
    );
  }
  const length = lengthInSecondsOf(lease);
  const filledWidth = TRACK_WIDTH * Math.min(1, secondsLeft / length);
  return (
    <span className="inline-flex flex-none items-center gap-1.5 font-mono text-[11px] text-ink-2">
      <meter
        aria-label={dictionary.tasks.leaseLeft}
        min={0}
        max={length}
        value={secondsLeft}
        className="sr-only"
      />
      <svg
        aria-hidden={true}
        width={TRACK_WIDTH}
        height={TRACK_HEIGHT}
        className="flex-none [shape-rendering:crispEdges]"
      >
        <rect
          className="fill-sunken"
          width={TRACK_WIDTH}
          height={TRACK_HEIGHT}
        />
        <rect
          className={fillOf(secondsLeft)}
          width={filledWidth}
          height={TRACK_HEIGHT}
        />
        <rect
          fill={`url(#${LEASE_DITHER})`}
          width={filledWidth}
          height={TRACK_HEIGHT}
        />
        <rect
          className="fill-none stroke-line"
          x={HALF_A_HAIRLINE}
          y={HALF_A_HAIRLINE}
          width={TRACK_WIDTH - HAIRLINE}
          height={TRACK_HEIGHT - HAIRLINE}
        />
      </svg>
      {dictionary.tasks.timeLeft(secondsLeft)}
    </span>
  );
}
