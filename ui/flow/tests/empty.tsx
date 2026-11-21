import {
  act,
  cleanup,
  getDefaultNormalizer,
  render,
  screen,
} from "@testing-library/react";
import { afterEach, expect, test } from "vitest";
import { dictionary } from "../../dictionary";
import { EmptyBoard } from "../empty";

afterEach(cleanup);

const BOARD_URL = "http://127.0.0.1:4200/api/board";

test("the notice gives a prompt for an agent that names the board", () => {
  render(<EmptyBoard boardUrl={BOARD_URL} />);

  expect(
    screen.getByText(dictionary.emptyBoard.prompt(BOARD_URL), {
      normalizer: getDefaultNormalizer({ collapseWhitespace: false }),
    }),
  ).toBeDefined();
  expect(screen.getByText(dictionary.emptyBoard.explanation)).toBeDefined();
});

test("the prompt for an agent copies with one click, and says it did", async () => {
  const copied: string[] = [];
  Object.defineProperty(navigator, "clipboard", {
    configurable: true,
    value: {
      writeText: (text: string) => {
        copied.push(text);
        return Promise.resolve();
      },
    },
  });
  render(<EmptyBoard boardUrl={BOARD_URL} />);

  await act(async () => {
    screen.getByRole("button", { name: dictionary.emptyBoard.copy }).click();
    await Promise.resolve();
  });

  expect(copied).toEqual([dictionary.emptyBoard.prompt(BOARD_URL)]);
  expect(
    screen.getByRole("button", { name: dictionary.emptyBoard.copied }),
  ).toBeDefined();
});
