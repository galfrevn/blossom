import { dictionary } from "../dictionary";
import type { Lens } from "./choice";

export type LensEntry = {
  lens: Lens;
  name: string;
  key: string;
};

export const LENSES: LensEntry[] = [
  { lens: "flow", name: dictionary.lenses.flow.title, key: "2" },
  { lens: "tasks", name: dictionary.lenses.tasks.title, key: "3" },
];
