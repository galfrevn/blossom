import { act, cleanup, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { createClock } from "../../core/clock";
import type { Lease } from "../../core/contract";
import { dictionary } from "../../dictionary";
import { LeaseLeft } from "../lease";

const NOW = Date.parse("2026-11-21T10:30:00.000Z");
const ONE_SECOND = 1000;
const SECONDS_LEFT = 65;
const LEASE_IN_SECONDS = 300;
const BOARD_DEFAULT_LEASE_IN_SECONDS = 900;
const TEN_SECONDS = 10 * ONE_SECOND;
const PAST_THE_END = (SECONDS_LEFT + 1) * ONE_SECOND;

function leaseOf(secondsLeft: number, lengthInSeconds: number): Lease {
  const startedSecondsAgo = lengthInSeconds - secondsLeft;
  return {
    since: new Date(NOW - startedSecondsAgo * ONE_SECOND).toISOString(),
    until: new Date(NOW + secondsLeft * ONE_SECOND).toISOString(),
  };
}

function renderLease(lease: Lease) {
  render(<LeaseLeft lease={lease} clock={createClock(ONE_SECOND)} />);
}

function letTimePass(milliseconds: number) {
  act(() => {
    vi.advanceTimersByTime(milliseconds);
  });
}

beforeEach(() => {
  vi.useFakeTimers({ toFake: ["setInterval", "clearInterval", "Date"] });
  vi.setSystemTime(NOW);
});

afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

test("a lease says how long it has left", () => {
  renderLease(leaseOf(SECONDS_LEFT, LEASE_IN_SECONDS));

  expect(screen.getByText("1:05")).toBeDefined();
});

test("a lease counts down as time passes, without asking the core", () => {
  renderLease(leaseOf(SECONDS_LEFT, LEASE_IN_SECONDS));

  letTimePass(TEN_SECONDS);

  expect(screen.getByText("0:55")).toBeDefined();
  expect(screen.queryByText("1:05")).toBeNull();
});

test("a lease reads as lapsed once past", () => {
  renderLease(leaseOf(SECONDS_LEFT, LEASE_IN_SECONDS));

  letTimePass(PAST_THE_END);

  expect(screen.getByText(dictionary.tasks.lapsed)).toBeDefined();
  expect(screen.queryByRole("meter")).toBeNull();
});

test("a lease's bar measures what is left against the whole lease", () => {
  renderLease(leaseOf(SECONDS_LEFT, LEASE_IN_SECONDS));

  const bar = screen.getByRole("meter", { name: dictionary.tasks.leaseLeft });

  expect(bar.getAttribute("value")).toBe(String(SECONDS_LEFT));
  expect(bar.getAttribute("max")).toBe(String(LEASE_IN_SECONDS));
});

test("a lease whose start the replica never saw is measured against the lease Board gives by default", () => {
  renderLease({ ...leaseOf(SECONDS_LEFT, LEASE_IN_SECONDS), since: null });

  const bar = screen.getByRole("meter", { name: dictionary.tasks.leaseLeft });

  expect(bar.getAttribute("max")).toBe(String(BOARD_DEFAULT_LEASE_IN_SECONDS));
});
