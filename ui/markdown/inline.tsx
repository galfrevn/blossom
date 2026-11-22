import type { MarkedToken, Token, Tokens } from "marked";
import type { ReactNode } from "react";
import { isPastTheDeepestNesting } from "./depth";
import { linkToOpen } from "./link";
import { drawnInOrder, markedTokensOf } from "./tokens";

const LINK_LOOK =
  "text-ink underline decoration-line-strong underline-offset-2 hover:decoration-ink";

type Nesting = Tokens.Strong | Tokens.Em | Tokens.Del | Tokens.Link;

function drawCheckbox(checked: boolean): ReactNode {
  return (
    <input
      type="checkbox"
      checked={checked}
      disabled={true}
      className="mr-[5px] -ml-[18px] size-[13px] align-[-2px] accent-ink-2"
    />
  );
}

function linkTo(
  href: string,
  title: string | null | undefined,
  name: ReactNode,
): ReactNode {
  return (
    <a href={href} title={title ?? href} className={LINK_LOOK}>
      {name}
    </a>
  );
}

function drawLink(token: Tokens.Link, name: ReactNode): ReactNode {
  const href = linkToOpen(token.href);
  if (href === undefined) {
    return name;
  }
  return linkTo(href, token.title, name);
}

function drawImage(token: Tokens.Image, depth: number): ReactNode {
  const description = elementsOfInline(token.tokens, depth);
  const href = linkToOpen(token.href);
  if (href === undefined) {
    return description;
  }
  if (token.tokens.length === 0) {
    return linkTo(href, token.title, href);
  }
  return linkTo(href, token.title, description);
}

function drawNesting(token: Nesting, depth: number): ReactNode {
  const nestedDepth = depth + 1;
  if (isPastTheDeepestNesting(nestedDepth)) {
    return token.raw;
  }
  const inside = elementsOfInline(token.tokens, nestedDepth);
  switch (token.type) {
    case "strong":
      return <strong className="font-semibold text-ink">{inside}</strong>;
    case "em":
      return <em>{inside}</em>;
    case "del":
      return <del>{inside}</del>;
    case "link":
      return drawLink(token, inside);
    default: {
      const nestingWithoutElement: never = token;
      return nestingWithoutElement;
    }
  }
}

function drawText(token: Tokens.Text, depth: number): ReactNode {
  if (token.tokens === undefined) {
    return token.text;
  }
  return elementsOfInline(token.tokens, depth);
}

function drawInline(token: MarkedToken, depth: number): ReactNode {
  switch (token.type) {
    case "text":
      return drawText(token, depth);
    case "escape":
    case "html":
      return token.text;
    case "codespan":
      return (
        <code className="rounded-[3px] border border-line bg-sunken px-1 font-mono text-[11.5px] text-ink">
          {token.text}
        </code>
      );
    case "br":
      return <br />;
    case "strong":
    case "em":
    case "del":
    case "link":
      return drawNesting(token, depth);
    case "image":
      return drawImage(token, depth);
    case "checkbox":
      return drawCheckbox(token.checked);
    default:
      return token.raw;
  }
}

export function elementsOfInline(tokens: Token[], depth: number): ReactNode[] {
  return drawnInOrder(markedTokensOf(tokens), (token) =>
    drawInline(token, depth),
  );
}
