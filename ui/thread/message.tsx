import { memo, type ReactNode } from "react";
import type { ThreadMessage } from "../core/contract";
import { dictionary } from "../dictionary";
import { elementsOfMarkdown } from "../markdown/blocks";

function bodyOf(message: ThreadMessage): ReactNode {
  if (message.deleted) {
    return dictionary.thread.deleted;
  }
  return elementsOfMarkdown(message.text);
}

function OnBehalfOf({ person }: { person: string | null }) {
  if (person === null) {
    return null;
  }
  return (
    <small className="text-ink-3">{dictionary.thread.onBehalfOf(person)}</small>
  );
}

function Edited({ message }: { message: ThreadMessage }) {
  if (!message.edited) {
    return null;
  }
  return (
    <span className="text-[11px] text-ink-3">{dictionary.thread.edited}</span>
  );
}

function MessageOfTheThread({ message }: { message: ThreadMessage }) {
  return (
    <article
      data-message={message.id}
      className="flex flex-col gap-[3px] border-line border-b py-2.5"
    >
      <header className="flex flex-wrap items-baseline gap-x-2 gap-y-0.5 text-[12.5px]">
        <span className="font-medium">{message.author}</span>
        <OnBehalfOf person={message.onBehalfOf} />
        <Edited message={message} />
        <time
          dateTime={message.createdAt}
          className="ml-auto font-mono text-[10.5px] text-ink-3"
        >
          {dictionary.thread.time(message.createdAt)}
        </time>
      </header>
      <div className="flex flex-col gap-1 text-ink-2 [overflow-wrap:anywhere]">
        {bodyOf(message)}
      </div>
    </article>
  );
}

function readsTheSame(
  previous: { message: ThreadMessage },
  next: { message: ThreadMessage },
): boolean {
  const before = previous.message;
  const after = next.message;
  return (
    before.id === after.id &&
    before.author === after.author &&
    before.onBehalfOf === after.onBehalfOf &&
    before.createdAt === after.createdAt &&
    before.text === after.text &&
    before.edited === after.edited &&
    before.deleted === after.deleted
  );
}

export const Message = memo(MessageOfTheThread, readsTheSame);
