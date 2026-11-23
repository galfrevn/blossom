import type { FlowRow } from "../core/contract";
import { dictionary } from "../dictionary";
import { namesOf } from "../markdown/mention";
import { plainTextOf } from "../markdown/plain";

export function textOf(row: FlowRow): string {
  if (row.deleted) {
    return dictionary.flow.deleted;
  }
  return plainTextOf(row.text, namesOf(row.mentions));
}
