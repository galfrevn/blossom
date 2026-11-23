import { expect, test } from "vitest";
import { matchOf } from "../matching";

test("with nothing typed, every title matches well and nothing is marked", () => {
  expect(matchOf("Go to Tasks", "  ")).toEqual({
    quality: "well",
    marked: [{ text: "Go to Tasks", marked: false }],
  });
});

test("words typed that start words of the title match well, and are marked", () => {
  expect(matchOf("Go to Tasks", "go ta")).toEqual({
    quality: "well",
    marked: [
      { text: "Go", marked: true },
      { text: " to ", marked: false },
      { text: "Ta", marked: true },
      { text: "sks", marked: false },
    ],
  });
});

test("case does not matter", () => {
  expect(matchOf("Show the whole board", "WHOLE")?.quality).toBe("well");
});

test("a word typed found only inside a word of the title matches weakly", () => {
  expect(matchOf("Go to Flow", "low")).toEqual({
    quality: "weakly",
    marked: [
      { text: "Go to F", marked: false },
      { text: "low", marked: true },
    ],
  });
});

test("one word typed that is not in the title leaves it out", () => {
  expect(matchOf("Go to Tasks", "go deploy")).toBeUndefined();
});

test("quotes typed for the search's phrases are not looked for in titles", () => {
  expect(matchOf("Show the whole board", '"whole board"')?.quality).toBe(
    "well",
  );
});
