import { useCallback, useState } from "react";
import type { FlowChange, FlowWindow, Thread } from "../core/contract";
import { dictionary } from "../dictionary";
import { LensHeader } from "../frame/lens";
import { OpenThread } from "../thread/open";
import { type ThreadOpenings, useThreadOpenings } from "../thread/openings";
import { FlowList } from "./list";

export function FlowLens({
  change,
  channel,
  boardUrl,
  onWiden,
  readFlow,
  countFlowBefore,
  countFlowAfter,
  readThread,
  openLink,
  threadOpenings,
}: {
  change: FlowChange | undefined;
  channel: string | null;
  boardUrl: string;
  onWiden: () => void;
  readFlow: (
    channel: string | null,
    start: number,
    count: number,
  ) => Promise<FlowWindow>;
  countFlowBefore: (
    channel: string | null,
    createdAt: string,
    id: string,
  ) => Promise<number>;
  countFlowAfter: (
    channel: string | null,
    createdAt: string,
    id: string,
  ) => Promise<number>;
  readThread: (messageId: string) => Promise<Thread | null>;
  openLink: (link: string) => Promise<void>;
  threadOpenings: ThreadOpenings;
}) {
  const [selectedMessage, setSelectedMessage] = useState<string>();
  useThreadOpenings(threadOpenings, setSelectedMessage);
  const closeTheThread = useCallback(() => {
    setSelectedMessage(undefined);
  }, []);
  return (
    <>
      <main className="flex min-w-0 flex-1 flex-col">
        <LensHeader
          title={dictionary.lenses.flow.title}
          description={dictionary.lenses.flow.description}
          channel={channel}
          onWiden={onWiden}
        />
        <FlowList
          key={channel ?? ""}
          change={change}
          channel={channel}
          boardUrl={boardUrl}
          readFlow={readFlow}
          countFlowBefore={countFlowBefore}
          countFlowAfter={countFlowAfter}
          selectedMessage={selectedMessage}
          onSelect={setSelectedMessage}
        />
      </main>
      <OpenThread
        messageId={selectedMessage}
        change={change}
        readThread={readThread}
        openLink={openLink}
        onClose={closeTheThread}
      />
    </>
  );
}
