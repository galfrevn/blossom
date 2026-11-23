import { Lexer, type Token } from "marked";
import { mentionsNamedBy, type Names } from "./mention";

export function tokensOf(markdown: string, names: Names): Token[] | undefined {
  try {
    return Lexer.lex(markdown, {
      gfm: true,
      breaks: true,
      extensions: mentionsNamedBy(names),
    });
  } catch (error: unknown) {
    console.error("blossom: a message could not be read as markdown", error);
    return undefined;
  }
}
