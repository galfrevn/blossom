import type { OpenBoard } from "../core/contract";
import { dictionary } from "../dictionary";

type BoardThatIsOpen = Extract<OpenBoard, { kind: "open" }>;

export function StatusBar({ openBoard }: { openBoard: BoardThatIsOpen }) {
  return (
    <footer className="flex h-[26px] shrink-0 items-center justify-between border-t border-line px-3 text-[11px] text-ink-2">
      <span className="font-mono">{openBoard.url}</span>
      <span className="flex gap-1.5">
        <span className="text-ink">{openBoard.person.name}</span>
        <span className="text-ink-3">
          {dictionary.status.handle(openBoard.person.handle)}
        </span>
      </span>
    </footer>
  );
}
