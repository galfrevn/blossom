import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, expect, test } from "vitest";
import type { ReplicaState } from "../../core/contract";
import { StatusBar } from "../status";

afterEach(cleanup);

function statusBarWhileTheReplicaIs(replicaState: ReplicaState) {
  render(
    <StatusBar
      openBoard={{
        kind: "open",
        url: "http://127.0.0.1:4100/api/board",
        person: { handle: "ada", name: "Ada Lovelace" },
      }}
      replicaState={replicaState}
    />,
  );
  return within(screen.getByRole("contentinfo"));
}

test("while the board is copied, the status bar says how far along the copy is", () => {
  const statusBar = statusBarWhileTheReplicaIs({
    kind: "copying",
    stored: 42_000,
    total: 100_006,
  });

  expect(statusBar.getByText("Copying 42,000 of 100,006")).toBeDefined();
});

test("when the board does not say its total, the status bar says how many are copied so far", () => {
  const statusBar = statusBarWhileTheReplicaIs({
    kind: "copying",
    stored: 42_000,
    total: null,
  });

  expect(statusBar.getByText("Copying 42,000")).toBeDefined();
});

test("a replica of one message says so in the singular", () => {
  const statusBar = statusBarWhileTheReplicaIs({ kind: "live", stored: 1 });

  expect(statusBar.getByText("1 message")).toBeDefined();
});

test("while the feed is reconnecting, the status bar says so", () => {
  const statusBar = statusBarWhileTheReplicaIs({
    kind: "reconnecting",
    stored: 100_006,
  });

  expect(statusBar.getByText("Reconnecting")).toBeDefined();
});

test("when the replica stops, the status bar says why", () => {
  const statusBar = statusBarWhileTheReplicaIs({
    kind: "stopped",
    stored: 100_006,
    reason: "unwritableReplica",
  });

  expect(statusBar.getByText("Stopped: can't write the replica")).toBeDefined();
});

test("when the board sends what the core cannot read, the status bar says so", () => {
  const statusBar = statusBarWhileTheReplicaIs({
    kind: "stopped",
    stored: 7,
    reason: "unreadableBoard",
  });

  expect(statusBar.getByText("Stopped: can't read the board")).toBeDefined();
});
