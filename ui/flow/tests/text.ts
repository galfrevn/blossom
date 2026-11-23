import { expect, test } from "vitest";
import type { FlowRow } from "../../core/contract";
import { dictionary } from "../../dictionary";
import { textOf } from "../text";

function rowSaying(overrides: Partial<FlowRow>): FlowRow {
  return {
    id: "1",
    channel: "fleet/www",
    author: "claude",
    onBehalfOf: "ada",
    createdAt: "2026-11-21T10:00:00.000Z",
    text: "deploy failed",
    mentions: [],
    edited: false,
    deleted: false,
    firstOfItsMinute: false,
    fold: null,
    ...overrides,
  };
}

test("a line reads its message's markdown as plain text, and a mention as the name of whom it mentions", () => {
  const row = rowSaying({
    text: "## Deploy\n\n**failed** on `www`, see [the run](https://vercel.com/runs/1), @[local:user:ada+codex]",
    mentions: [{ id: "local:user:ada+codex", name: "codex" }],
  });

  expect(textOf(row)).toBe("Deploy\nfailed on www, see the run, @codex");
});

test("a deleted message reads as deleted, whatever its text was", () => {
  const row = rowSaying({ text: "**deploy www**", deleted: true });

  expect(textOf(row)).toBe(dictionary.flow.deleted);
});
