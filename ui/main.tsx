import "./theme/tailwind.css";
import { isTauri } from "@tauri-apps/api/core";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { Application } from "./application";
import type { Core } from "./core/client";
import { createClock } from "./core/clock";
import { createFakeCore } from "./core/fake";
import { createStore } from "./core/store";
import { createTauriCore } from "./core/tauri";

const ONE_SECOND = 1000;

function coreForThisWindow(): Core {
  if (isTauri()) {
    return createTauriCore();
  }
  const fakeCore = createFakeCore();
  fakeCore.pushOpenBoard({ kind: "noneRunning" });
  return fakeCore.core;
}

const rootElement = document.querySelector("#root");
if (rootElement === null) {
  throw new Error("index.html has no element with the id root");
}

const core = coreForThisWindow();
const openBoardStore = createStore(core.subscribeToOpenBoard);
const replicaStore = createStore(core.subscribeToReplica);
const channelsStore = createStore(core.subscribeToChannels);
const flowStore = createStore(core.subscribeToFlow);
const tasksStore = createStore(core.subscribeToTasks);
const clock = createClock(ONE_SECOND);

createRoot(rootElement).render(
  <StrictMode>
    <Application
      openBoardStore={openBoardStore}
      replicaStore={replicaStore}
      channelsStore={channelsStore}
      flowStore={flowStore}
      tasksStore={tasksStore}
      clock={clock}
      readFlow={core.readFlow}
      countFlowBefore={core.countFlowBefore}
      countFlowAfter={core.countFlowAfter}
      readThread={core.readThread}
      openLink={core.openLink}
      readTasks={core.readTasks}
      readTaskCounts={core.readTaskCounts}
      search={core.search}
    />
  </StrictMode>,
);
