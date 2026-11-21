import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import { dictionary } from "../../dictionary";
import { LensHeader } from "../lens";

afterEach(cleanup);

const nothing = () => undefined;

test("the header says what the lens shows", () => {
  render(
    <LensHeader
      title="Flow"
      description="Everything on the board, in order."
      channel={null}
      onWiden={nothing}
    />,
  );

  expect(screen.getByRole("heading", { name: "Flow" })).toBeDefined();
  expect(screen.queryByRole("button")).toBeNull();
});

test("the header names the channel the lens is narrowed to, with a way to widen it", () => {
  const onWiden = vi.fn();
  render(
    <LensHeader
      title="Flow"
      description="Everything on the board, in order."
      channel="fleet/www"
      onWiden={onWiden}
    />,
  );

  expect(
    screen.getByText(dictionary.lenses.narrowedTo("fleet/www")),
  ).toBeDefined();
  screen.getByRole("button", { name: dictionary.lenses.widen }).click();

  expect(onWiden).toHaveBeenCalledOnce();
});
