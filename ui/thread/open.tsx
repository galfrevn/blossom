import type { FlowChange, Thread } from "../core/contract";
import { ThreadPane } from "./pane";

export function OpenThread({
  messageId,
  change,
  readThread,
  openLink,
  onClose,
}: {
  messageId: string | undefined;
  change: FlowChange | undefined;
  readThread: (messageId: string) => Promise<Thread | null>;
  openLink: (link: string) => Promise<void>;
  onClose: () => void;
}) {
  if (messageId === undefined) {
    return null;
  }
  return (
    <ThreadPane
      messageId={messageId}
      change={change}
      readThread={readThread}
      openLink={openLink}
      onClose={onClose}
    />
  );
}
