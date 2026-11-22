import { cleanup, render } from "@testing-library/react";
import { afterEach, expect, test } from "vitest";
import { elementsOfMarkdown } from "../blocks";
import { DEEPEST_NESTING } from "../depth";

afterEach(cleanup);

const LEVELS_PAST_THE_DEEPEST = 4;
const LONGEST_MESSAGE = 10_000;
const EMPHASIS_LEVELS_TRIED = DEEPEST_NESTING * 2;
const STARS_PER_EMPHASIS_LEVEL = 2;

function renderMarkdown(markdown: string): HTMLElement {
  const { container } = render(<div>{elementsOfMarkdown(markdown)}</div>);
  const text = container.firstElementChild;
  if (!(text instanceof HTMLElement)) {
    throw new Error("the markdown was not rendered into its container");
  }
  return text;
}

test("a quote nested past the deepest level reads as written from there on", () => {
  const levels = DEEPEST_NESTING + LEVELS_PAST_THE_DEEPEST;
  const text = renderMarkdown(`${"> ".repeat(levels)}rolled back`);

  expect(text.querySelectorAll("blockquote")).toHaveLength(DEEPEST_NESTING);
  expect(text.textContent).toBe(
    `${"> ".repeat(LEVELS_PAST_THE_DEEPEST)}rolled back`,
  );
});

test("a list nested past the deepest level reads as written from there on", () => {
  const levels = DEEPEST_NESTING + LEVELS_PAST_THE_DEEPEST;
  const markdown = Array.from(
    { length: levels },
    (_, level) => `${"  ".repeat(level)}- step ${level}`,
  ).join("\n");

  const text = renderMarkdown(markdown);

  expect(text.querySelectorAll("ul")).toHaveLength(DEEPEST_NESTING);
  expect(text.textContent).toContain(`- step ${levels - 1}`);
});

test("quotes and lists inside each other count as one nesting", () => {
  const text = renderMarkdown(`${"- > ".repeat(DEEPEST_NESTING)}deep`);

  expect(text.querySelectorAll("ul, blockquote")).toHaveLength(DEEPEST_NESTING);
  expect(text.textContent).toContain("deep");
});

test("a message that is nothing but quotes, as long as a message can be, draws", () => {
  const text = renderMarkdown(">".repeat(LONGEST_MESSAGE));

  expect(text.querySelectorAll("blockquote").length).toBeLessThanOrEqual(
    DEEPEST_NESTING,
  );
  expect(text.textContent).toContain(
    ">".repeat(LONGEST_MESSAGE - DEEPEST_NESTING),
  );
});

test("emphasis nested past the deepest level reads as written from there on", () => {
  const stars = "*".repeat(EMPHASIS_LEVELS_TRIED * STARS_PER_EMPHASIS_LEVEL);
  const text = renderMarkdown(`${stars}deep${stars}`);

  expect(text.querySelectorAll("em, strong").length).toBeLessThanOrEqual(
    DEEPEST_NESTING,
  );
  expect(text.textContent).toContain("deep");
});

test("a message that is nothing but emphasis, as long as a message can be, draws", () => {
  const stars = "*".repeat(LONGEST_MESSAGE / 2);
  const text = renderMarkdown(`${stars}deep${stars}`);

  expect(text.querySelectorAll("em, strong").length).toBeLessThanOrEqual(
    DEEPEST_NESTING,
  );
  expect(text.textContent).toContain("deep");
});
