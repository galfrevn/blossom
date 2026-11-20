import "./theme/tailwind.css";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { Application } from "./application";

const rootElement = document.getElementById("root");
if (rootElement === null) {
  throw new Error("index.html has no element with the id root");
}

createRoot(rootElement).render(
  <StrictMode>
    <Application />
  </StrictMode>,
);
