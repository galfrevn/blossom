import type { TextPart } from "../core/contract";

type Quality = "well" | "weakly";

type Found = {
  start: number;
  end: number;
  atWordStart: boolean;
};

const WORD = /\S+/gu;
const QUOTES = /"/gu;
const SPACES = /\s+/u;

function wordsTyped(query: string): string[] {
  return query
    .replace(QUOTES, " ")
    .toLowerCase()
    .split(SPACES)
    .filter((word) => word !== "");
}

function foundIn(title: string, word: string): Found | undefined {
  const lowerTitle = title.toLowerCase();
  for (const titleWord of title.matchAll(WORD)) {
    if (lowerTitle.startsWith(word, titleWord.index)) {
      const start = titleWord.index;
      return { start, end: start + word.length, atWordStart: true };
    }
  }
  const start = lowerTitle.indexOf(word);
  if (start === -1) {
    return undefined;
  }
  return { start, end: start + word.length, atWordStart: false };
}

function partsOf(title: string, found: Found[]): TextPart[] {
  const isMarked = (index: number) =>
    found.some((span) => span.start <= index && index < span.end);
  const parts: TextPart[] = [];
  for (const [index, character] of [...title].entries()) {
    const marked = isMarked(index);
    const last = parts.at(-1);
    if (last?.marked === marked) {
      last.text += character;
    } else {
      parts.push({ text: character, marked });
    }
  }
  return parts;
}

function qualityOf(found: Found[]): Quality {
  if (found.every((span) => span.atWordStart)) {
    return "well";
  }
  return "weakly";
}

export type Match = {
  quality: Quality;
  marked: TextPart[];
};

export function matchOf(title: string, query: string): Match | undefined {
  const found: Found[] = [];
  for (const word of wordsTyped(query)) {
    const span = foundIn(title, word);
    if (span === undefined) {
      return undefined;
    }
    found.push(span);
  }
  return { quality: qualityOf(found), marked: partsOf(title, found) };
}
