import { expect, test } from "vitest";
import { linkToOpen } from "../link";

const ALERT = "alert(1)";
const SCRIPT = `<script>${ALERT}</script>`;

test("web and mail links open, written as the browser reads them", () => {
  expect(linkToOpen("https://vercel.com/docs")).toBe("https://vercel.com/docs");
  expect(linkToOpen("http://www.vercel.com")).toBe("http://www.vercel.com/");
  expect(linkToOpen("HTTPS://VERCEL.COM")).toBe("https://vercel.com/");
  expect(linkToOpen("mailto:ada@example.com")).toBe("mailto:ada@example.com");
});

test("a link that would run something or reach into the machine never opens", () => {
  const dangerousLinks = [
    "javascript:alert(1)",
    "JavaScript:alert(1)",
    " javascript:alert(1)",
    "java\tscript:alert(1)",
    "vbscript:msgbox(1)",
    `data:text/html,${SCRIPT}`,
    "file:///Applications/Calculator.app",
    "blob:https://vercel.com/1",
    "tauri://localhost/",
  ];

  for (const link of dangerousLinks) {
    expect(linkToOpen(link), link).toBeUndefined();
  }
});

test("a link with no scheme of its own never opens", () => {
  for (const link of ["", "/threads/2", "#top", "vercel.com", "./report"]) {
    expect(linkToOpen(link), link).toBeUndefined();
  }
});
