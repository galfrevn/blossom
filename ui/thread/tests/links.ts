import { cleanup, fireEvent, renderHook } from "@testing-library/react";
import { afterEach, expect, test } from "vitest";
import { createFakeCore } from "../../core/fake";
import { useOpeningLinksInTheBrowser } from "../links";

const MIDDLE_BUTTON = 1;
const RIGHT_BUTTON = 2;

afterEach(() => {
  cleanup();
  document.body.replaceChildren();
});

function holderOfLinkAndWords() {
  const holder = document.createElement("section");
  const link = document.createElement("a");
  link.href = "https://vercel.com/docs";
  link.textContent = "docs";
  const words = document.createElement("span");
  words.textContent = "plain words";
  holder.append(link, words);
  document.body.append(holder);
  return { holder, link, words };
}

function renderLinks() {
  const fakeCore = createFakeCore();
  const { holder, link, words } = holderOfLinkAndWords();
  renderHook(() =>
    useOpeningLinksInTheBrowser({ current: holder }, fakeCore.core.openLink),
  );
  return { link, words, linksOpened: fakeCore.linksOpened };
}

function auxiliaryClickWith(button: number): MouseEvent {
  return new MouseEvent("auxclick", {
    bubbles: true,
    cancelable: true,
    button,
  });
}

test("a middle click on a link opens it in the browser, and the window stays where it is", () => {
  const { link, linksOpened } = renderLinks();

  const theWindowWouldFollowTheLink = fireEvent(
    link,
    auxiliaryClickWith(MIDDLE_BUTTON),
  );

  expect(theWindowWouldFollowTheLink).toBe(false);
  expect(linksOpened()).toEqual(["https://vercel.com/docs"]);
});

test("a command click on a link opens it in the browser, and the window stays where it is", () => {
  const { link, linksOpened } = renderLinks();

  const theWindowWouldFollowTheLink = fireEvent.click(link, { metaKey: true });

  expect(theWindowWouldFollowTheLink).toBe(false);
  expect(linksOpened()).toEqual(["https://vercel.com/docs"]);
});

test("a right click on a link opens nothing", () => {
  const { link, linksOpened } = renderLinks();

  fireEvent(link, auxiliaryClickWith(RIGHT_BUTTON));

  expect(linksOpened()).toEqual([]);
});

test("a middle click beside a link opens nothing", () => {
  const { words, linksOpened } = renderLinks();

  fireEvent(words, auxiliaryClickWith(MIDDLE_BUTTON));

  expect(linksOpened()).toEqual([]);
});
