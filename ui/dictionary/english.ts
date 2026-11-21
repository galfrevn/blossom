const countFormat = new Intl.NumberFormat("en-US");
const clockTime = new Intl.DateTimeFormat("en-GB", {
  hour: "2-digit",
  minute: "2-digit",
  hourCycle: "h23",
});

function count(value: number): string {
  return countFormat.format(value);
}

function plural(value: number, one: string, many: string): string {
  if (value === 1) {
    return `1 ${one}`;
  }
  return `${count(value)} ${many}`;
}

export const english = {
  start: {
    title: "No board running",
    instruction: "Start one in a project's checkout with",
    afterwards: "Blossom opens it as soon as it starts.",
  },
  emptyBoard: {
    title: "No agent has written on this board yet",
    explanation:
      "Blossom shows a board; it does not run agents. Point one at this board and its claims, progress and questions appear here as they happen.",
    prompt: (url: string) =>
      `Use the Vercel Board at ${url} with the vb CLI.\nWork the tasks in #general: claim one, post your progress in its thread,\nand when you need a decision from me, block the task and ask in the thread.`,
    where: "Paste this into Claude Code, Codex or any agent",
    copy: "Copy",
    copied: "Copied",
  },
  sidebar: {
    lenses: "Lenses",
    channels: "Channels",
    channel: (name: string) => `#${name}`,
  },
  lenses: {
    narrowedTo: (channel: string) => `in ${channel}`,
    widen: "Show the whole board",
    widenMark: "×",
    flow: {
      title: "Flow",
      description:
        "Everything on the board, in order. Repeated progress folds into one line per channel; open a line to read it.",
    },
  },
  flow: {
    onBehalfOf: (person: string) => `via ${person}`,
    channel: (channel: string) => `#${channel}`,
    time: (createdAt: string) => clockTime.format(new Date(createdAt)),
    deleted: "Deleted",
    edited: "edited",
    foldCount: (updates: number, agents: number) => {
      const updatesText = plural(
        updates,
        "progress update",
        "progress updates",
      );
      const agentsText = plural(agents, "agent", "agents");
      return `${updatesText} from ${agentsText}`;
    },
    latest: (text: string) => `latest: ${text}`,
    newBelow: (waiting: number) => {
      if (waiting === 1) {
        return "1 new message";
      }
      return `${count(waiting)} new messages`;
    },
  },
  thread: {
    title: "Thread",
    inChannel: (channel: string) => `Thread in #${channel}`,
    close: "Close",
    closeKey: "esc",
    onBehalfOf: (person: string) => `via ${person}`,
    time: (createdAt: string) => clockTime.format(new Date(createdAt)),
    deleted: "Deleted",
    edited: "edited",
  },
  status: {
    handle: (handle: string) => `@${handle}`,
    copying: (stored: number, total: number | null) => {
      if (total === null) {
        return `Copying ${count(stored)}`;
      }
      return `Copying ${count(stored)} of ${count(total)}`;
    },
    live: (stored: number) => {
      if (stored === 1) {
        return "1 message";
      }
      return `${count(stored)} messages`;
    },
    reconnecting: "Reconnecting",
    stopped: {
      unreadableBoard: "Stopped: can't read the board",
      unwritableReplica: "Stopped: can't write the replica",
    },
  },
};
