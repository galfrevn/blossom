import { expect, test } from "vitest";
import type { FlowRow } from "../../core/contract";
import { NewMessagesBelow } from "../below";

function rowNumbered(id: string): FlowRow {
  return {
    id,
    channel: "fleet/www",
    author: "claude",
    onBehalfOf: "ada",
    createdAt: "2026-11-21T10:00:00.000Z",
    text: "work",
    edited: false,
    deleted: false,
    firstOfItsMinute: false,
    fold: null,
  };
}

const LINES_AFTER_THE_SECOND = 3;
const LINES_AFTER_THE_FIFTH = 1;

const settle = () => new Promise((resolve) => setTimeout(resolve, 0));
const nothing = () => undefined;

function coreThatCountsByHand() {
  const asked: string[] = [];
  const unanswered: Array<(count: number) => void> = [];
  const countAfter = (_createdAt: string, id: string): Promise<number> =>
    new Promise((resolve) => {
      asked.push(id);
      unanswered.push(resolve);
    });
  const answerTheOldest = async (count: number) => {
    unanswered.shift()?.(count);
    await settle();
  };
  return { asked, countAfter, answerTheOldest };
}

test("it counts the lines after the newest one the reader saw before leaving the bottom", async () => {
  const core = coreThatCountsByHand();
  let counted = 0;
  const below = new NewMessagesBelow(core.countAfter, () => {
    counted += 1;
  });

  below.leaveTheBottomAt(rowNumbered("2"));
  below.recount();
  await core.answerTheOldest(LINES_AFTER_THE_SECOND);

  expect(core.asked).toEqual(["2"]);
  expect(below.count()).toBe(LINES_AFTER_THE_SECOND);
  expect(counted).toBe(1);
});

test("leaving the bottom again starts the count again, and an answer about the earlier time is dropped", async () => {
  const core = coreThatCountsByHand();
  const below = new NewMessagesBelow(core.countAfter, nothing);
  below.leaveTheBottomAt(rowNumbered("2"));
  below.recount();
  await core.answerTheOldest(LINES_AFTER_THE_SECOND);

  below.leaveTheBottomAt(rowNumbered("5"));
  below.recount();
  expect(below.count()).toBe(0);
  below.leaveTheBottomAt(rowNumbered("6"));
  await core.answerTheOldest(LINES_AFTER_THE_FIFTH);

  expect(below.count()).toBe(0);
});

test("with no line seen before leaving the bottom, nothing is counted", async () => {
  const core = coreThatCountsByHand();
  const below = new NewMessagesBelow(core.countAfter, nothing);

  below.leaveTheBottomAt(undefined);
  below.recount();
  await settle();

  expect(core.asked).toEqual([]);
  expect(below.count()).toBe(0);
});
