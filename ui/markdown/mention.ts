import type {
  MarkedOptions,
  Token,
  TokenizerExtensionFunction,
  TokenizerStartFunction,
} from "marked";
import type { MentionedActor } from "../core/contract";

const MENTION = /^@\[(?<id>[^[\]\s]+)\]/u;
const MENTION_START = "@[";

type LexerExtensions = NonNullable<MarkedOptions["extensions"]>;

function readingOf(id: string, names: Names): string {
  return `@${names.get(id) ?? id}`;
}

export type Names = ReadonlyMap<string, string>;

export type MentionToken = {
  type: "mention";
  raw: string;
  text: string;
};

export function namesOf(mentions: MentionedActor[]): Names {
  return new Map(mentions.map((actor) => [actor.id, actor.name]));
}

export function isMention(token: Token): token is Token & MentionToken {
  return token.type === "mention";
}

export function mentionsNamedBy(names: Names): LexerExtensions {
  const mention: TokenizerExtensionFunction = (source) => {
    const found = MENTION.exec(source);
    const id = found?.groups?.id;
    if (found === null || id === undefined) {
      return;
    }
    const token: MentionToken = {
      type: "mention",
      raw: found[0],
      text: readingOf(id, names),
    };
    return token;
  };
  const whereMentionsStart: TokenizerStartFunction = (source) => {
    const index = source.indexOf(MENTION_START);
    if (index === -1) {
      return;
    }
    return index;
  };
  return {
    renderers: {},
    childTokens: {},
    inline: [mention],
    startInline: [whereMentionsStart],
  };
}
