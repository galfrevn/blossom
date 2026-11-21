import type { OpenBoard, ReplicaState } from "../core/contract";
import { dictionary } from "../dictionary";

type BoardThatIsOpen = Extract<OpenBoard, { kind: "open" }>;

function describeReplica(replicaState: ReplicaState): string {
  if (replicaState.kind === "copying") {
    return dictionary.status.copying(replicaState.stored, replicaState.total);
  }
  return dictionary.status.copied(replicaState.stored);
}

export function StatusBar({
  openBoard,
  replicaState,
}: {
  openBoard: BoardThatIsOpen;
  replicaState: ReplicaState | undefined;
}) {
  return (
    <footer className="flex h-[26px] shrink-0 items-center justify-between gap-6 border-t border-line px-3 text-[11px] text-ink-2">
      <span className="min-w-0 truncate font-mono">{openBoard.url}</span>
      <span className="flex min-w-0 max-w-[60%] items-center gap-4">
        {replicaState !== undefined && (
          <span className="shrink-0 tabular-nums text-ink-3">
            {describeReplica(replicaState)}
          </span>
        )}
        <span className="flex min-w-0 gap-1.5">
          <span className="min-w-0 truncate text-ink">
            {openBoard.person.name}
          </span>
          <span className="shrink-0 text-ink-3">
            {dictionary.status.handle(openBoard.person.handle)}
          </span>
        </span>
      </span>
    </footer>
  );
}
