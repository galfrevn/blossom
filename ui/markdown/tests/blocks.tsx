import { cleanup, render, within } from "@testing-library/react";
import { afterEach, expect, test } from "vitest";
import { elementsOfMarkdown } from "../blocks";

afterEach(cleanup);

const HEADING_UNDER_THE_PANE = 3;
const DEEPEST_HEADING = 6;
const START_OF_THE_NUMBERED_LIST = "3";
const ALERT = "alert(1)";
const SCRIPT = `<script>${ALERT}</script>`;
const NOT_EMPHASIS = "*not emphasis*";
const BANNER = `<div class="banner">\n${NOT_EMPHASIS}\n</div>`;
const SUMMARY = "logs";
const DETAILS = `<details><summary>${SUMMARY}</summary>`;

function renderMarkdown(markdown: string) {
  const { container } = render(
    <div>{elementsOfMarkdown(markdown, new Map())}</div>,
  );
  const text = container.firstElementChild;
  if (!(text instanceof HTMLElement)) {
    throw new Error("the markdown was not rendered into its container");
  }
  return { text, view: within(text) };
}

function onlyElement(text: HTMLElement, selector: string): HTMLElement {
  const [element, ...others] = text.querySelectorAll<HTMLElement>(selector);
  if (element === undefined || others.length > 0) {
    throw new Error(`there is not exactly one ${selector}`);
  }
  return element;
}

test("a paragraph keeps its words, and a single newline breaks its line", () => {
  const { text } = renderMarkdown("deploy failed\nrolled back");

  const paragraph = onlyElement(text, "p");

  expect(paragraph.textContent).toBe("deploy failedrolled back");
  expect(paragraph.querySelectorAll("br")).toHaveLength(1);
});

test("a blank line starts a paragraph of its own", () => {
  const { text } = renderMarkdown("deploy failed\n\nrolled back");

  const paragraphs = [...text.querySelectorAll("p")];

  expect(paragraphs.map((paragraph) => paragraph.textContent)).toEqual([
    "deploy failed",
    "rolled back",
  ]);
});

test("headings sit under the pane's own heading, one level for each #", () => {
  const { view } = renderMarkdown(
    "# Report\n\n## Build\n\n### Tests\n\n###### Detail",
  );

  expect(
    view.getByRole("heading", {
      level: HEADING_UNDER_THE_PANE,
      name: "Report",
    }),
  ).toBeDefined();
  expect(
    view.getByRole("heading", {
      level: HEADING_UNDER_THE_PANE + 1,
      name: "Build",
    }),
  ).toBeDefined();
  expect(
    view.getByRole("heading", {
      level: HEADING_UNDER_THE_PANE + 2,
      name: "Tests",
    }),
  ).toBeDefined();
  expect(
    view.getByRole("heading", { level: DEEPEST_HEADING, name: "Detail" }),
  ).toBeDefined();
});

test("a bulleted list keeps its items in order", () => {
  const { view } = renderMarkdown("- build\n- test\n- deploy");

  const list = view.getByRole("list");

  expect(list.tagName).toBe("UL");
  expect(
    within(list)
      .getAllByRole("listitem")
      .map((item) => item.textContent),
  ).toEqual(["build", "test", "deploy"]);
});

test("a numbered list keeps the number it starts from", () => {
  const { view } = renderMarkdown("3. three\n4. four");

  const list = view.getByRole("list");

  expect(list.tagName).toBe("OL");
  expect(list.getAttribute("start")).toBe(START_OF_THE_NUMBERED_LIST);
});

test("a list inside an item stays inside it", () => {
  const { view } = renderMarkdown("- www\n  - build\n  - deploy\n- api");

  const [outer, inner] = view.getAllByRole("list");

  expect(outer?.contains(inner ?? null)).toBe(true);
  expect(
    within(inner as HTMLElement)
      .getAllByRole("listitem")
      .map((item) => item.textContent),
  ).toEqual(["build", "deploy"]);
});

test("a task list shows whether each item is done, and nobody can tick it", () => {
  const { view } = renderMarkdown("- [x] build\n- [ ] deploy");

  const [built, deployed] = view.getAllByRole("checkbox");

  expect(built).toHaveProperty("checked", true);
  expect(deployed).toHaveProperty("checked", false);
  expect(built).toHaveProperty("disabled", true);
  expect(deployed).toHaveProperty("disabled", true);
  expect(view.getByText("build")).toBeDefined();
});

test("a task list whose items are apart still shows whether each is done", () => {
  const { view } = renderMarkdown("- [x] build\n\n- [ ] deploy");

  const boxes = view.getAllByRole("checkbox");

  expect(boxes.map((box) => (box as HTMLInputElement).checked)).toEqual([
    true,
    false,
  ]);
});

test("a quote reads as a quote", () => {
  const { text } = renderMarkdown("> rolled back\n> to d4e5f6");

  const quote = onlyElement(text, "blockquote");

  expect(quote.textContent).toBe("rolled backto d4e5f6");
});

test("a rule divides the text", () => {
  const { view } = renderMarkdown("before\n\n---\n\nafter");

  expect(view.getByRole("separator")).toBeDefined();
});

test("a table keeps its header, its rows and the alignment of each column", () => {
  const { view } = renderMarkdown(
    "| package | tests |\n| :-- | --: |\n| www | 1208 |\n| api | 96 |",
  );

  const table = view.getByRole("table");
  const headers = within(table).getAllByRole("columnheader");
  const cells = within(table).getAllByRole("cell");

  expect(headers.map((header) => header.textContent)).toEqual([
    "package",
    "tests",
  ]);
  expect(cells.map((cell) => cell.textContent)).toEqual([
    "www",
    "1208",
    "api",
    "96",
  ]);
  expect(cells[1]?.classList.contains("text-right")).toBe(true);
  expect(cells[0]?.classList.contains("text-left")).toBe(true);
});

test("a fenced code block keeps its spacing and names its language", () => {
  const { text, view } = renderMarkdown(
    "```ts\nconst attempts = 3;\n\n    retry();\n```",
  );

  const code = onlyElement(text, "pre code");

  expect(code.textContent).toBe("const attempts = 3;\n\n    retry();");
  expect(view.getByText("ts")).toBeDefined();
});

test("a code block names only the first word of its fence", () => {
  const { view } = renderMarkdown("```py title=retry.py\nretry()\n```");

  expect(view.getByText("py")).toBeDefined();
  expect(view.queryByText("title=retry.py")).toBeNull();
});

test("a code block with no language names none", () => {
  const { text } = renderMarkdown("```\nls -la\n```");

  expect(onlyElement(text, "pre code").textContent).toBe("ls -la");
  expect(text.querySelector("figcaption")).toBeNull();
});

test("an indented code block is a code block too", () => {
  const { text } = renderMarkdown("Trace:\n\n    at retry (www.ts:12)\n");

  expect(onlyElement(text, "pre code").textContent).toBe(
    "at retry (www.ts:12)",
  );
});

test("a code block wraps a long line, as the concept draws it, and keeps its spacing", () => {
  const { text } = renderMarkdown(
    "```\nError: a line far longer than the pane\n```",
  );

  const block = onlyElement(text, "pre");

  expect(block.classList.contains("whitespace-pre-wrap")).toBe(true);
});

test("a code block's text can be selected and copied", () => {
  const { text } = renderMarkdown("```\nexit 1\n```");

  const block = onlyElement(text, "pre");

  expect(block.classList.contains("select-text")).toBe(true);
});

test("a wide table scrolls sideways rather than squeezing its words", () => {
  const { view } = renderMarkdown("| a | b |\n| - | - |\n| 1 | 2 |");

  const table = view.getByRole("table");

  expect(table.parentElement?.classList.contains("overflow-x-auto")).toBe(true);
});

test("raw HTML reads as the text it is, and makes no element of its own", () => {
  const { text, view } = renderMarkdown(BANNER);

  expect(text.querySelector(".banner")).toBeNull();
  expect(text.querySelector("em")).toBeNull();
  expect(
    view.getByText(BANNER, {
      normalizer: String,
    }),
  ).toBeDefined();
});

test("a script in a message is text, and never becomes a script", () => {
  const { text, view } = renderMarkdown(`${SCRIPT}\n\nafter`);

  expect(text.querySelector("script")).toBeNull();
  expect(view.getByText(SCRIPT)).toBeDefined();
  expect(view.getByText("after")).toBeDefined();
});

test("HTML that hides what follows hides nothing", () => {
  const { text, view } = renderMarkdown(
    `${DETAILS}\n\nthe whole log\n\n</details>`,
  );

  expect(text.querySelector("details")).toBeNull();
  expect(view.getByText("the whole log")).toBeDefined();
});

test("a script inside a code block is code", () => {
  const { text } = renderMarkdown(`\`\`\`html\n${SCRIPT}\n\`\`\``);

  expect(text.querySelector("script")).toBeNull();
  expect(onlyElement(text, "pre code").textContent).toBe(SCRIPT);
});

test("a link's definition draws nothing of its own", () => {
  const { text, view } = renderMarkdown(
    "See [the report][report].\n\n[report]: https://vercel.com/report",
  );

  expect(view.getByRole("link", { name: "the report" })).toBeDefined();
  expect(text.textContent).toBe("See the report.");
});

test("a code block inside a quote inside a list keeps all three", () => {
  const { text } = renderMarkdown(
    "- deploy\n  > it failed with\n  > ```sh\n  > exit 1\n  > ```",
  );

  const code = onlyElement(text, "li blockquote pre code");

  expect(code.textContent).toBe("exit 1");
});

test("an empty message draws nothing", () => {
  const { text } = renderMarkdown("");

  expect(text.childElementCount).toBe(0);
});
