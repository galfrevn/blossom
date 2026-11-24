import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, expect, test } from "vitest";
import type { ThreadMessage } from "../../core/contract";
import { dictionary } from "../../dictionary";
import { Message } from "../message";

afterEach(cleanup);

const SOMEONE_ACTING_FOR_A_PERSON = /^via /u;
const BOLD_WWW = "<b>www</b>";
const SCRIPT = "<script>x</script>";

function renderMessage(overrides: Partial<ThreadMessage>) {
  const message: ThreadMessage = {
    id: "root",
    author: "claude",
    onBehalfOf: "ada",
    createdAt: "2026-11-21T10:00:00.000Z",
    text: "deploy www",
    mentions: [],
    edited: false,
    deleted: false,
    ...overrides,
  };
  render(<Message message={message} landed={false} />);
  return screen.getByRole("article");
}

test("a message says who wrote it, on whose behalf, and when", () => {
  const message = renderMessage({ createdAt: "2026-11-21T10:07:00.000Z" });

  expect(within(message).getByText("claude")).toBeDefined();
  expect(
    within(message).getByText(dictionary.thread.onBehalfOf("ada")),
  ).toBeDefined();
  expect(
    within(message).getByText(
      dictionary.thread.time("2026-11-21T10:07:00.000Z"),
    ),
  ).toBeDefined();
});

test("a person's own message names no one it acts for", () => {
  const message = renderMessage({ author: "Ada Lovelace", onBehalfOf: null });

  expect(within(message).queryByText(SOMEONE_ACTING_FOR_A_PERSON)).toBeNull();
});

test("a message's text reads as markdown", () => {
  const message = renderMessage({
    text: "## Report\n\n- **built** www\n- ran `vb serve`",
  });

  expect(
    within(message).getByRole("heading", { name: "Report" }),
  ).toBeDefined();
  expect(within(message).getByText("built").tagName).toBe("STRONG");
  expect(within(message).getByText("vb serve").tagName).toBe("CODE");
  expect(within(message).getAllByRole("listitem")).toHaveLength(2);
});

test("raw HTML in a message reads as the text it is", () => {
  const message = renderMessage({ text: `built ${BOLD_WWW} ${SCRIPT}` });

  expect(message.querySelector("b")).toBeNull();
  expect(message.querySelector("script")).toBeNull();
  expect(
    within(message).getByText(`built ${BOLD_WWW} ${SCRIPT}`),
  ).toBeDefined();
});

test("a deleted message reads as deleted, whatever its text was", () => {
  const message = renderMessage({ deleted: true, text: "**deploy www**" });

  expect(within(message).getByText(dictionary.thread.deleted)).toBeDefined();
  expect(message.querySelector("strong")).toBeNull();
});

test("an edited message says so", () => {
  const message = renderMessage({ edited: true, text: "fixed" });

  expect(within(message).getByText(dictionary.thread.edited)).toBeDefined();
});

test("a mention reads as the name of whom it mentions", () => {
  const message = renderMessage({
    text: "@[local:user:ada+codex] can you look?",
    mentions: [{ id: "local:user:ada+codex", name: "codex" }],
  });

  expect(message.textContent).toContain("@codex can you look?");
  expect(within(message).queryByRole("link")).toBeNull();
});
