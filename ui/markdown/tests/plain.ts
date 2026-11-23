import { expect, test } from "vitest";
import { CHARACTERS_A_LINE_READS, plainTextOf } from "../plain";

const NOBODY = new Map<string, string>();
const CLAUDE = "local:user:ada+claude";
const NAMES = new Map([[CLAUDE, "claude"]]);
const WORDS_IN_A_LONG_MESSAGE = 2000;

test("a heading reads as its words, without its hashes", () => {
  expect(plainTextOf("## Deploy report", NOBODY)).toBe("Deploy report");
});

test("emphasis, strong emphasis and strikethrough read as their words", () => {
  const text = plainTextOf(
    "*retrying* **failed** ~~deployed~~ ***both***, snake_case_names and 2 * 3 * 4",
    NOBODY,
  );

  expect(text).toBe(
    "retrying failed deployed both, snake_case_names and 2 * 3 * 4",
  );
});

test("inline code reads as its text", () => {
  expect(plainTextOf("run `vb serve` again", NOBODY)).toBe(
    "run vb serve again",
  );
});

test("a fenced block reads as the code it holds, without its fences or its language", () => {
  const text = plainTextOf(
    "the fix:\n\n```ts\nconst answer = await fetch(url);\n```",
    NOBODY,
  );

  expect(text).toBe("the fix:\nconst answer = await fetch(url);");
});

test("a link reads as its text and an image as its description, without where they point", () => {
  const text = plainTextOf(
    'See [the run](https://vercel.com/runs/1 "Run 1") and ![the graph](https://vercel.com/graph.png).',
    NOBODY,
  );

  expect(text).toBe("See the run and the graph.");
});

test("a bare address reads as itself", () => {
  expect(plainTextOf("Preview at https://www.vercel.app", NOBODY)).toBe(
    "Preview at https://www.vercel.app",
  );
});

test("a quote, a list and a table read as their words, a line each", () => {
  const text = plainTextOf(
    "> rolled back\n\n- [x] built www\n- ran **tests**\n\n| step | took |\n| --- | --- |\n| build | 12 s |",
    NOBODY,
  );

  expect(text).toBe("rolled back\nbuilt www\nran tests\nstep took\nbuild 12 s");
});

test("lines that break stay apart, and each paragraph starts a line", () => {
  const text = plainTextOf(
    "first line\nsecond line\n\n\n\nnext paragraph\n\n---\n\nafter the rule",
    NOBODY,
  );

  expect(text).toBe("first line\nsecond line\nnext paragraph\nafter the rule");
});

test("raw HTML reads as the text it is", () => {
  expect(plainTextOf("built <b>www</b> &amp; api", NOBODY)).toBe(
    "built <b>www</b> &amp; api",
  );
});

test("a mention reads as the name of whom it mentions, and one nobody knows as its ID", () => {
  const text = plainTextOf(
    `over to **@[${CLAUDE}]**, cc @[local:user:nobody]`,
    NAMES,
  );

  expect(text).toBe("over to @claude, cc @local:user:nobody");
});

test("a mention in code reads as written", () => {
  expect(plainTextOf(`type \`@[${CLAUDE}]\``, NAMES)).toBe(`type @[${CLAUDE}]`);
});

test("a long message is read only as far as a line can show, and says it goes on", () => {
  const longMessage = "**word** ".repeat(WORDS_IN_A_LONG_MESSAGE);

  const text = plainTextOf(longMessage, NOBODY);

  expect(text.length).toBeLessThanOrEqual(CHARACTERS_A_LINE_READS);
  expect(text.startsWith("word word word")).toBe(true);
  expect(text.endsWith("…")).toBe(true);
});
