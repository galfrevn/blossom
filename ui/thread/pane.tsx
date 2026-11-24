import { useEffect, useId, useLayoutEffect, useRef, useState } from "react";
import type { FlowChange, Thread } from "../core/contract";
import { dictionary } from "../dictionary";
import { isAtTheBottom } from "../flow/state";
import { useLandingOnTheMessageAskedFor } from "./landing";
import { useOpeningLinksInTheBrowser } from "./links";
import { Message } from "./message";

type ThreadPaneProps = {
  messageId: string;
  change: FlowChange | undefined;
  readThread: (messageId: string) => Promise<Thread | null>;
  openLink: (link: string) => Promise<void>;
  onClose: () => void;
};

type ThreadRead = {
  thread: Thread;
  keepsToTheEnd: boolean;
};

type PaneBody = React.RefObject<HTMLDivElement | null>;

function isAtTheEnd(body: HTMLElement | null): boolean {
  if (body === null) {
    return false;
  }
  return isAtTheBottom(body);
}

function isTheSameThread(
  previous: ThreadRead | undefined,
  thread: Thread,
): boolean {
  return previous?.thread.id === thread.id;
}

function useThreadOf(props: ThreadPaneProps, body: PaneBody) {
  const { messageId, change, readThread, onClose } = props;
  const [read, setRead] = useState<ThreadRead | undefined>();
  useEffect(() => {
    if (change === undefined) {
      return;
    }
    let isTheLatestRead = true;
    readThread(messageId).then(
      (thread) => {
        if (!isTheLatestRead) {
          return;
        }
        if (thread === null) {
          onClose();
          return;
        }
        const wasAtTheEnd = isAtTheEnd(body.current);
        setRead((previous) => ({
          thread,
          keepsToTheEnd: wasAtTheEnd && isTheSameThread(previous, thread),
        }));
      },
      (error: unknown) => {
        console.error("blossom: a thread could not be read", error);
      },
    );
    return () => {
      isTheLatestRead = false;
    };
  }, [messageId, change, readThread, onClose, body]);
  return read;
}

function useKeepingToTheEnd(body: PaneBody, read: ThreadRead | undefined) {
  useLayoutEffect(() => {
    const element = body.current;
    if (element === null || read?.keepsToTheEnd !== true) {
      return;
    }
    element.scrollTop = element.scrollHeight;
  }, [body, read]);
}

function useClosingOnEscape(onClose: () => void) {
  useEffect(() => {
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onClose();
      }
    };
    document.addEventListener("keydown", closeOnEscape);
    return () => {
      document.removeEventListener("keydown", closeOnEscape);
    };
  }, [onClose]);
}

function headingOf(read: ThreadRead | undefined): string {
  if (read === undefined) {
    return dictionary.thread.title;
  }
  return dictionary.thread.inChannel(read.thread.channel);
}

function Messages({
  read,
  landed,
}: {
  read: ThreadRead | undefined;
  landed: string | undefined;
}) {
  if (read === undefined) {
    return null;
  }
  return read.thread.messages.map((message) => (
    <Message
      key={message.id}
      message={message}
      landed={message.id === landed}
    />
  ));
}

function CloseButton({ onClose }: { onClose: () => void }) {
  return (
    <button
      type="button"
      onClick={onClose}
      aria-label={dictionary.thread.close}
      aria-keyshortcuts="Escape"
      className="inline-flex items-center gap-2 whitespace-nowrap rounded-md border border-transparent px-2.5 py-1 font-medium text-[12px] text-ink-2 hover:bg-sunken hover:text-ink"
    >
      {dictionary.thread.close}
      <kbd className="rounded-[3px] border border-line bg-raised px-1 py-0.5 font-mono text-[10px] text-ink-3 leading-none">
        {dictionary.thread.closeKey}
      </kbd>
    </button>
  );
}

export function ThreadPane(props: ThreadPaneProps) {
  const pane = useRef<HTMLElement>(null);
  const body = useRef<HTMLDivElement>(null);
  const headingId = useId();
  const read = useThreadOf(props, body);
  useKeepingToTheEnd(body, read);
  const landed = useLandingOnTheMessageAskedFor(
    body,
    read?.thread,
    props.messageId,
  );
  useClosingOnEscape(props.onClose);
  useOpeningLinksInTheBrowser(pane, props.openLink);
  return (
    <aside
      ref={pane}
      aria-labelledby={headingId}
      className="flex min-h-0 w-[min(380px,36vw)] shrink-0 flex-col border-line border-l bg-raised"
    >
      <header className="flex min-h-[38px] items-center justify-between gap-2.5 border-line border-b py-1.5 pr-2.5 pl-4 font-mono text-[11px] text-ink-3">
        <h2 id={headingId} className="min-w-0 truncate">
          {headingOf(read)}
        </h2>
        <CloseButton onClose={props.onClose} />
      </header>
      <div
        ref={body}
        key={read?.thread.id}
        role="feed"
        aria-labelledby={headingId}
        className="flex min-h-0 flex-1 flex-col overflow-auto px-4 pt-4 pb-5"
      >
        <Messages read={read} landed={landed} />
      </div>
    </aside>
  );
}
