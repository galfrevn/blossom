import { useEffect, useLayoutEffect, useRef, useState } from "react";
import type { Thread } from "../core/contract";

type PaneBody = React.RefObject<HTMLElement | null>;

function articleOf(body: HTMLElement, messageId: string): Element | null {
  return body.querySelector(`[data-message="${CSS.escape(messageId)}"]`);
}

function isTheRootOf(thread: Thread, messageId: string): boolean {
  return thread.messages[0]?.id === messageId;
}

function useMarkThatFades(): [string | undefined, (messageId: string) => void] {
  const [marked, setMarked] = useState<string | undefined>(undefined);
  useEffect(() => {
    if (marked === undefined) {
      return;
    }
    const fading = setTimeout(() => setMarked(undefined), MARK_LASTS);
    return () => clearTimeout(fading);
  }, [marked]);
  return [marked, setMarked];
}

export const MARK_LASTS = 1600;

export function useLandingOnTheMessageAskedFor(
  body: PaneBody,
  thread: Thread | undefined,
  messageId: string,
): string | undefined {
  const landedOn = useRef<string>(undefined);
  const [marked, mark] = useMarkThatFades();
  useLayoutEffect(() => {
    const element = body.current;
    if (thread === undefined || element === null) {
      return;
    }
    if (landedOn.current === messageId) {
      return;
    }
    if (isTheRootOf(thread, messageId)) {
      landedOn.current = messageId;
      return;
    }
    const article = articleOf(element, messageId);
    if (article === null) {
      return;
    }
    landedOn.current = messageId;
    article.scrollIntoView({ block: "center" });
    mark(messageId);
  }, [body, thread, messageId, mark]);
  return marked;
}
