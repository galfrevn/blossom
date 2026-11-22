import { Lexer, type MarkedToken, type Token, type Tokens } from "marked";
import type { ReactNode } from "react";
import { isPastTheDeepestNesting } from "./depth";
import { elementsOfInline } from "./inline";
import {
  drawnInOrder,
  markedTokensOf,
  withoutTrailingNewlines,
} from "./tokens";

const FIRST_WORD = /^\S+/u;
const BLOCKS_APART = "flex flex-col gap-1";
const ITEM_LOOK = "[&>*+*]:mt-1";

const HEADING_TAGS = ["h3", "h4", "h5", "h6"] as const;
const HEADING_SIZES = ["text-[14px]", "text-[13px]"] as const;
const SMALLEST_HEADING_SIZE = "text-[12.5px]";

const CELL_ALIGNMENTS = {
  left: "text-left",
  center: "text-center",
  right: "text-right",
} as const;

type Alignment = Tokens.TableCell["align"];

function writtenAsIs(text: string): ReactNode {
  return <p className="whitespace-pre-wrap">{withoutTrailingNewlines(text)}</p>;
}

function drawHeading(token: Tokens.Heading, depth: number): ReactNode {
  const level = Math.min(token.depth, HEADING_TAGS.length) - 1;
  const HeadingTag = HEADING_TAGS[level] ?? "h6";
  const size = HEADING_SIZES[level] ?? SMALLEST_HEADING_SIZE;
  return (
    <HeadingTag className={`mt-1 font-semibold text-ink first:mt-0 ${size}`}>
      {elementsOfInline(token.tokens, depth)}
    </HeadingTag>
  );
}

function languageOf(token: Tokens.Code): string | undefined {
  return FIRST_WORD.exec(token.lang ?? "")?.[0];
}

function drawLanguage(language: string | undefined): ReactNode {
  if (language === undefined) {
    return null;
  }
  return (
    <figcaption className="border-line border-b px-2.5 py-1 font-mono text-[10.5px] text-ink-3 leading-none">
      {language}
    </figcaption>
  );
}

function drawCode(token: Tokens.Code): ReactNode {
  return (
    <figure className="m-0 overflow-hidden rounded-[5px] border border-line bg-paper">
      {drawLanguage(languageOf(token))}
      <pre className="m-0 cursor-text select-text whitespace-pre-wrap px-2.5 py-2 font-mono text-[11.5px] text-ink leading-[1.5] [overflow-wrap:anywhere]">
        <code>{withoutTrailingNewlines(token.text)}</code>
      </pre>
    </figure>
  );
}

function drawQuote(token: Tokens.Blockquote, depth: number): ReactNode {
  const nestedDepth = depth + 1;
  if (isPastTheDeepestNesting(nestedDepth)) {
    return writtenAsIs(token.raw);
  }
  return (
    <blockquote
      className={`${BLOCKS_APART} border-line-strong border-l-2 pl-2.5 text-ink-3`}
    >
      {elementsOfBlocks(token.tokens, nestedDepth)}
    </blockquote>
  );
}

function lookOfItem(item: Tokens.ListItem): string {
  if (item.task) {
    return `${ITEM_LOOK} list-none`;
  }
  return ITEM_LOOK;
}

function drawListItem(item: Tokens.ListItem, depth: number): ReactNode {
  return (
    <li className={lookOfItem(item)}>{elementsOfBlocks(item.tokens, depth)}</li>
  );
}

function startOf(token: Tokens.List): number | undefined {
  if (token.start === "") {
    return undefined;
  }
  return token.start;
}

function drawList(token: Tokens.List, depth: number): ReactNode {
  const nestedDepth = depth + 1;
  if (isPastTheDeepestNesting(nestedDepth)) {
    return writtenAsIs(token.raw);
  }
  const items = drawnInOrder(token.items, (item) =>
    drawListItem(item, nestedDepth),
  );
  if (token.ordered) {
    return (
      <ol
        start={startOf(token)}
        className="list-decimal space-y-0.5 pl-[22px] marker:text-ink-3"
      >
        {items}
      </ol>
    );
  }
  return (
    <ul className="list-disc space-y-0.5 pl-[18px] marker:text-ink-3">
      {items}
    </ul>
  );
}

function alignmentOf(alignment: Alignment): string {
  if (alignment === null) {
    return CELL_ALIGNMENTS.left;
  }
  return CELL_ALIGNMENTS[alignment];
}

function drawHeaderCell(cell: Tokens.TableCell, depth: number): ReactNode {
  return (
    <th
      className={`border border-line bg-sunken px-2 py-1 font-medium text-ink ${alignmentOf(cell.align)}`}
    >
      {elementsOfInline(cell.tokens, depth)}
    </th>
  );
}

function drawCell(cell: Tokens.TableCell, depth: number): ReactNode {
  return (
    <td className={`border border-line px-2 py-1 ${alignmentOf(cell.align)}`}>
      {elementsOfInline(cell.tokens, depth)}
    </td>
  );
}

function drawTable(token: Tokens.Table, depth: number): ReactNode {
  const header = drawnInOrder(token.header, (cell) =>
    drawHeaderCell(cell, depth),
  );
  const rows = drawnInOrder(token.rows, (row) => (
    <tr>{drawnInOrder(row, (cell) => drawCell(cell, depth))}</tr>
  ));
  return (
    <div className="overflow-x-auto">
      <table className="border-collapse text-[12px] tabular-nums [overflow-wrap:normal]">
        <thead>
          <tr>{header}</tr>
        </thead>
        <tbody>{rows}</tbody>
      </table>
    </div>
  );
}

function drawBlock(token: MarkedToken, depth: number): ReactNode {
  switch (token.type) {
    case "paragraph":
      return <p>{elementsOfInline(token.tokens, depth)}</p>;
    case "heading":
      return drawHeading(token, depth);
    case "code":
      return drawCode(token);
    case "blockquote":
      return drawQuote(token, depth);
    case "list":
      return drawList(token, depth);
    case "table":
      return drawTable(token, depth);
    case "hr":
      return <hr className="my-1 border-line" />;
    case "html":
      return writtenAsIs(token.raw);
    case "space":
    case "def":
      return null;
    default:
      return elementsOfInline([token], depth);
  }
}

function elementsOfBlocks(tokens: Token[], depth: number): ReactNode[] {
  return drawnInOrder(markedTokensOf(tokens), (token) =>
    drawBlock(token, depth),
  );
}

function tokensOf(markdown: string): Token[] | undefined {
  try {
    return Lexer.lex(markdown, { gfm: true, breaks: true });
  } catch (error: unknown) {
    console.error("blossom: a message could not be read as markdown", error);
    return undefined;
  }
}

export function elementsOfMarkdown(markdown: string): ReactNode {
  const tokens = tokensOf(markdown);
  if (tokens === undefined) {
    return writtenAsIs(markdown);
  }
  return elementsOfBlocks(tokens, 0);
}
