import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, test } from "vitest";
import { TitleBar } from "../title";

afterEach(cleanup);

test("the window is dragged by its title bar", () => {
  render(<TitleBar>{null}</TitleBar>);

  const titleBar = screen.getByRole("banner");

  expect(titleBar.hasAttribute("data-tauri-drag-region")).toBe(true);
});
