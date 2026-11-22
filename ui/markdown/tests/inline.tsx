import { cleanup, render, within } from "@testing-library/react";
import { afterEach, expect, test } from "vitest";
import { elementsOfMarkdown } from "../blocks";

afterEach(cleanup);

const ALERT = "alert(1)";
const IMAGE_THAT_ALERTS = `<img src="x" onerror="${ALERT}">`;

function renderMarkdown(markdown: string) {
  const { container } = render(<div>{elementsOfMarkdown(markdown)}</div>);
  const text = container.firstElementChild;
  if (!(text instanceof HTMLElement)) {
    throw new Error("the markdown was not rendered into its container");
  }
  return { text, view: within(text) };
}

function textOfEach(text: HTMLElement, selector: string): (string | null)[] {
  return [...text.querySelectorAll(selector)].map(
    (element) => element.textContent,
  );
}

test("emphasis, strong emphasis and strikethrough each read as such", () => {
  const { text } = renderMarkdown(
    "*retrying* **failed** ~~deployed~~ ***both***",
  );

  expect(textOfEach(text, "em")).toEqual(["retrying", "both"]);
  expect(textOfEach(text, "strong")).toEqual(["failed", "both"]);
  expect(textOfEach(text, "del")).toEqual(["deployed"]);
});

test("inline code keeps its text, markup characters and all", () => {
  const { text } = renderMarkdown("run `<b>vb</b> & *serve*` again");

  expect(textOfEach(text, "code")).toEqual(["<b>vb</b> & *serve*"]);
  expect(text.querySelector("b")).toBeNull();
});

test("a link reads as its text and points where it was written to", () => {
  const { view } = renderMarkdown(
    'See [the deploy](https://vercel.com/deploys/1 "Deploy 1").',
  );

  const link = view.getByRole("link", { name: "the deploy" });

  expect(link.getAttribute("href")).toBe("https://vercel.com/deploys/1");
  expect(link.getAttribute("title")).toBe("Deploy 1");
});

test("bare web addresses and emails become links", () => {
  const { view } = renderMarkdown(
    "Preview at https://www-git-fix.vercel.app, docs at www.vercel.com, ask ada@example.com",
  );

  const hrefs = view
    .getAllByRole("link")
    .map((link) => link.getAttribute("href"));

  expect(hrefs).toEqual([
    "https://www-git-fix.vercel.app/",
    "http://www.vercel.com/",
    "mailto:ada@example.com",
  ]);
});

test("a link's text keeps its own markdown", () => {
  const { view } = renderMarkdown("[`vb serve` docs](https://vercel.com/docs)");

  const link = view.getByRole("link", { name: "vb serve docs" });

  expect(link.querySelector("code")?.textContent).toBe("vb serve");
});

test("a javascript: link is only its text", () => {
  const { text, view } = renderMarkdown(
    "[click me](javascript:alert(document.cookie))",
  );

  expect(view.queryByRole("link")).toBeNull();
  expect(text.textContent).toBe("click me");
});

test("links to data, files and places inside the app are only their text", () => {
  const { view } = renderMarkdown(
    "[a](data:text/html,x) [b](file:///etc/hosts) [c](/threads/2) [d](#top) [e](vbscript:x)",
  );

  expect(view.queryByRole("link")).toBeNull();
  expect(view.getByText("a b c d e")).toBeDefined();
});

test("an image is a link to it, and nothing loads by itself", () => {
  const { text, view } = renderMarkdown(
    "![the failing screen](https://vercel.com/screens/1.png)",
  );

  expect(text.querySelector("img")).toBeNull();
  expect(
    view.getByRole("link", { name: "the failing screen" }).getAttribute("href"),
  ).toBe("https://vercel.com/screens/1.png");
});

test("an image with no description is a link named by its address", () => {
  const { view } = renderMarkdown("![](https://vercel.com/screens/1.png)");

  expect(
    view.getByRole("link", { name: "https://vercel.com/screens/1.png" }),
  ).toBeDefined();
});

test("an image whose address would run something is only its description", () => {
  const { text, view } = renderMarkdown(`![screen](javascript:${ALERT})`);

  expect(view.queryByRole("link")).toBeNull();
  expect(text.textContent).toBe("screen");
});

test("raw HTML inside a line reads as written", () => {
  const { text } = renderMarkdown(`built <b>www</b> and ${IMAGE_THAT_ALERTS}`);

  expect(text.querySelector("b")).toBeNull();
  expect(text.querySelector("img")).toBeNull();
  expect(text.textContent).toBe(`built <b>www</b> and ${IMAGE_THAT_ALERTS}`);
});

test("an HTML link is text, not a link", () => {
  const { view } = renderMarkdown('<a href="https://vercel.com">vercel</a>');

  expect(view.queryByRole("link")).toBeNull();
});

test("escaped characters read as themselves", () => {
  const { text } = renderMarkdown("\\*not emphasis\\*");

  expect(text.querySelector("em")).toBeNull();
  expect(text.textContent).toBe("*not emphasis*");
});

test("a mention reads as it was written", () => {
  const { text } = renderMarkdown("over to @[local:user:ada]");

  expect(text.textContent).toBe("over to @[local:user:ada]");
  expect(text.querySelector("a")).toBeNull();
});

test("emphasis that never closes is text", () => {
  const { text } = renderMarkdown("**unfinished and `unclosed");

  expect(text.textContent).toBe("**unfinished and `unclosed");
});

test("a named character reference reads as written, as raw HTML does, and a numbered one as its character", () => {
  const { text } = renderMarkdown("AT&amp;T &lt;b&gt; &#169;");

  expect(text.textContent).toBe("AT&amp;T &lt;b&gt; \u00a9");
});

test("a link names where it goes when nothing else names it", () => {
  const { view } = renderMarkdown("[the deploy](https://vercel.com/deploys/1)");

  expect(view.getByRole("link").getAttribute("title")).toBe(
    "https://vercel.com/deploys/1",
  );
});
