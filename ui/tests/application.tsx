import { render, screen } from "@testing-library/react";
import { expect, test } from "vitest";
import { Application } from "../application";

test("the window is dragged by its title bar", () => {
  render(<Application />);

  const titleBar = screen.getByRole("banner");

  expect(titleBar.hasAttribute("data-tauri-drag-region")).toBe(true);
});
