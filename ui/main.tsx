import "./theme/tailwind.css";
import { isTauri } from "@tauri-apps/api/core";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { Application } from "./application";
import type { Core } from "./core/client";
import { createFakeCore } from "./core/fake";
import { createOpenBoardStore } from "./core/store";
import { createTauriCore } from "./core/tauri";

function coreForThisWindow(): Core {
  if (isTauri()) {
    return createTauriCore();
  }
  const fakeCore = createFakeCore();
  fakeCore.push({ kind: "noneRunning" });
  return fakeCore.core;
}

const rootElement = document.getElementById("root");
if (rootElement === null) {
  throw new Error("index.html has no element with the id root");
}

const openBoardStore = createOpenBoardStore(coreForThisWindow());

createRoot(rootElement).render(
  <StrictMode>
    <Application openBoardStore={openBoardStore} />
  </StrictMode>,
);
