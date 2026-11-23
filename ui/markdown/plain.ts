import type { Token, Tokens } from "marked";
import { isPastTheDeepestNesting } from "./depth";
import { tokensOf } from "./lexer";
import { isMention, type Names } from "./mention";
import { markedTokenOf, withoutTrailingNewlines } from "./tokens";

const GOES_ON = "…";
const WHITESPACE = /\s/u;

type Nesting = Tokens.Strong | Tokens.Em | Tokens.Del | Tokens.Link;

function inlineTextOf(tokens: Token[], depth: number): string {
  return tokens.map((token) => inlineOf(token, depth)).join("");
}

function nestedTextOf(token: Nesting, depth: number): string {
  const nestedDepth = depth + 1;
  if (isPastTheDeepestNesting(nestedDepth)) {
    return token.raw;
  }
  return inlineTextOf(token.tokens, nestedDepth);
}

function imageTextOf(token: Tokens.Image, depth: number): string {
  if (token.tokens.length === 0) {
    return token.href;
  }
  return inlineTextOf(token.tokens, depth);
}

function textOfText(token: Tokens.Text, depth: number): string {
  if (token.tokens === undefined) {
    return token.text;
  }
  return inlineTextOf(token.tokens, depth);
}

function inlineOf(token: Token, depth: number): string {
  if (isMention(token)) {
    return token.text;
  }
  const marked = markedTokenOf(token);
  switch (marked.type) {
    case "text":
      return textOfText(marked, depth);
    case "escape":
    case "html":
    case "codespan":
      return marked.text;
    case "br":
      return "\n";
    case "strong":
    case "em":
    case "del":
    case "link":
      return nestedTextOf(marked, depth);
    case "image":
      return imageTextOf(marked, depth);
    case "checkbox":
      return "";
    default:
      return marked.raw;
  }
}

function linesOf(tokens: Token[], depth: number): string {
  return tokens
    .map((token) => blockOf(token, depth).trim())
    .filter((line) => line !== "")
    .join("\n");
}

function nestedLinesOf(
  token: Tokens.Blockquote | Tokens.List,
  depth: number,
  readTheLines: (nestedDepth: number) => string,
): string {
  const nestedDepth = depth + 1;
  if (isPastTheDeepestNesting(nestedDepth)) {
    return token.raw;
  }
  return readTheLines(nestedDepth);
}

function tableTextOf(token: Tokens.Table, depth: number): string {
  const rowText = (cells: Tokens.TableCell[]) =>
    cells.map((cell) => inlineTextOf(cell.tokens, depth)).join(" ");
  return [rowText(token.header), ...token.rows.map(rowText)].join("\n");
}

function blockOf(token: Token, depth: number): string {
  const marked = markedTokenOf(token);
  switch (marked.type) {
    case "paragraph":
    case "heading":
      return inlineTextOf(marked.tokens, depth);
    case "code":
    case "html":
      return withoutTrailingNewlines(marked.text);
    case "blockquote":
      return nestedLinesOf(marked, depth, (nestedDepth) =>
        linesOf(marked.tokens, nestedDepth),
      );
    case "list":
      return nestedLinesOf(marked, depth, (nestedDepth) =>
        marked.items
          .map((item) => linesOf(item.tokens, nestedDepth))
          .join("\n"),
      );
    case "table":
      return tableTextOf(marked, depth);
    case "hr":
    case "space":
    case "def":
      return "";
    default:
      return inlineOf(token, depth);
  }
}

function startOf(markdown: string): { start: string; goesOn: boolean } {
  if (markdown.length <= CHARACTERS_A_LINE_READS) {
    return { start: markdown, goesOn: false };
  }
  const longest = markdown.slice(0, CHARACTERS_A_LINE_READS - GOES_ON.length);
  let end = longest.length;
  while (end > 0 && !WHITESPACE.test(longest.charAt(end - 1))) {
    end -= 1;
  }
  if (end === 0) {
    return { start: longest, goesOn: true };
  }
  return { start: longest.slice(0, end), goesOn: true };
}

function plainTextOfTheStart(start: string, names: Names): string {
  const tokens = tokensOf(start, names);
  if (tokens === undefined) {
    return start;
  }
  return linesOf(tokens, 0);
}

export const CHARACTERS_A_LINE_READS = 1000;

export function plainTextOf(markdown: string, names: Names): string {
  const { start, goesOn } = startOf(markdown);
  const text = plainTextOfTheStart(start, names);
  if (goesOn) {
    return `${text}${GOES_ON}`;
  }
  return text;
}
