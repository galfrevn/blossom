import type { MarkedToken, Token } from "marked";
import { Fragment, type ReactNode } from "react";

const TRAILING_NEWLINES = /\n+$/u;

export function markedTokensOf(tokens: Token[]): MarkedToken[] {
  return tokens as MarkedToken[];
}

export function markedTokenOf(token: Token): MarkedToken {
  return token as MarkedToken;
}

export function drawnInOrder<Item>(
  items: Item[],
  draw: (item: Item) => ReactNode,
): ReactNode[] {
  const drawn: ReactNode[] = [];
  for (const [position, item] of items.entries()) {
    drawn.push(<Fragment key={position}>{draw(item)}</Fragment>);
  }
  return drawn;
}

export function withoutTrailingNewlines(text: string): string {
  return text.replace(TRAILING_NEWLINES, "");
}
