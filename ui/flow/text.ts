import type { FlowRow } from "../core/contract";
import { dictionary } from "../dictionary";

export function textOf(row: FlowRow): string {
  if (row.deleted) {
    return dictionary.flow.deleted;
  }
  return row.text;
}
