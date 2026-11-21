function lowestBitOf(position: number): number {
  let bit = 1;
  while (position % (bit * 2) === 0) {
    bit *= 2;
  }
  return bit;
}

export class RowHeights {
  readonly count: number;
  private readonly estimate: number;
  private readonly heights: Float64Array;
  private readonly measured: Uint8Array;
  private readonly tree: Float64Array;

  constructor(
    count: number,
    estimate: number,
    earlier?: { heights: Float64Array; measured: Uint8Array },
  ) {
    this.count = count;
    this.estimate = estimate;
    this.heights = new Float64Array(count).fill(estimate);
    this.measured = new Uint8Array(count);
    if (earlier !== undefined) {
      const kept = Math.min(count, earlier.heights.length);
      this.heights.set(earlier.heights.subarray(0, kept));
      this.measured.set(earlier.measured.subarray(0, kept));
    }
    this.tree = new Float64Array(count + 1);
    for (let index = 0; index < count; index += 1) {
      this.tree[index + 1] =
        (this.tree[index + 1] ?? 0) + (this.heights[index] ?? 0);
      const parent = index + 1 + lowestBitOf(index + 1);
      if (parent <= count) {
        this.tree[parent] =
          (this.tree[parent] ?? 0) + (this.tree[index + 1] ?? 0);
      }
    }
  }

  heightOf(index: number): number {
    return this.heights[index] ?? this.estimate;
  }

  isMeasured(index: number): boolean {
    return this.measured[index] === 1;
  }

  forgetMeasurements(): void {
    this.measured.fill(0);
  }

  forgetMeasurementOf(index: number): void {
    if (index >= 0 && index < this.count) {
      this.measured[index] = 0;
    }
  }

  set(index: number, height: number): number {
    if (index < 0 || index >= this.count) {
      return 0;
    }
    this.measured[index] = 1;
    const change = height - this.heightOf(index);
    if (change === 0) {
      return 0;
    }
    this.heights[index] = height;
    for (let node = index + 1; node <= this.count; node += lowestBitOf(node)) {
      this.tree[node] = (this.tree[node] ?? 0) + change;
    }
    return change;
  }

  offsetOf(index: number): number {
    let sum = 0;
    for (
      let node = Math.min(index, this.count);
      node > 0;
      node -= lowestBitOf(node)
    ) {
      sum += this.tree[node] ?? 0;
    }
    return sum;
  }

  total(): number {
    return this.offsetOf(this.count);
  }

  rowAt(offset: number): number {
    let position = 0;
    let remaining = offset;
    let step = 1;
    while (step * 2 <= this.count) {
      step *= 2;
    }
    for (; step > 0; step = Math.floor(step / 2)) {
      const next = position + step;
      const nodeHeight = this.tree[next] ?? 0;
      if (next <= this.count && nodeHeight <= remaining) {
        position = next;
        remaining -= nodeHeight;
      }
    }
    return Math.max(0, Math.min(position, this.count - 1));
  }

  resized(count: number): RowHeights {
    return new RowHeights(count, this.estimate, {
      heights: this.heights,
      measured: this.measured,
    });
  }
}
