const countFormat = new Intl.NumberFormat("en-US");
const clockTime = new Intl.DateTimeFormat("en-GB", {
  hour: "2-digit",
  minute: "2-digit",
  hourCycle: "h23",
});
const dayOfTheMonth = new Intl.DateTimeFormat("en-GB", {
  day: "numeric",
  month: "short",
});

function count(value: number): string {
  return countFormat.format(value);
}

const SECONDS_IN_A_MINUTE = 60;
const SECONDS_IN_AN_HOUR = 3600;
const SECONDS_IN_A_DAY = 86_400;
const DIGITS_OF_A_PADDED_NUMBER = 2;

function padded(value: number): string {
  return String(value).padStart(DIGITS_OF_A_PADDED_NUMBER, "0");
}

function timeSince(seconds: number): string {
  if (seconds < SECONDS_IN_A_MINUTE) {
    return `${seconds} s`;
  }
  if (seconds < SECONDS_IN_AN_HOUR) {
    return `${Math.floor(seconds / SECONDS_IN_A_MINUTE)} min`;
  }
  if (seconds < SECONDS_IN_A_DAY) {
    const hours = Math.floor(seconds / SECONDS_IN_AN_HOUR);
    const minutes = Math.floor(
      (seconds % SECONDS_IN_AN_HOUR) / SECONDS_IN_A_MINUTE,
    );
    return `${hours} h ${padded(minutes)} min`;
  }
  return `${count(Math.floor(seconds / SECONDS_IN_A_DAY))} d`;
}

function timeLeft(seconds: number): string {
  const minutes = Math.floor(seconds / SECONDS_IN_A_MINUTE);
  return `${minutes}:${padded(seconds % SECONDS_IN_A_MINUTE)}`;
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
    openTasks: (openTasks: number) =>
      plural(openTasks, "open task", "open tasks"),
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
    tasks: {
      title: "Tasks",
      description:
        "Every task on the board, open ones first. Choose a state to see only its tasks, or filter them by title; open a row to read its thread.",
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
  tasks: {
    list: "Tasks",
    states: "States",
    stateNames: {
      blocked: "Blocked",
      failed: "Failed",
      claimed: "Claimed",
      ready: "Ready",
      waiting: "Waiting",
      completed: "Completed",
      canceled: "Canceled",
    },
    columns: {
      state: "State",
      task: "Task",
      channel: "Channel",
      owner: "Owner",
      heldBy: "Held by",
      updated: "Updated",
    },
    count,
    channel: (channel: string) => `#${channel}`,
    filter: "Filter by title",
    timeSince,
    timeLeft,
    leaseLeft: "Lease left",
    lapsed: "lapsed",
    noTasks: "No tasks on this board yet",
    noneMatch: "No task matches",
  },
  commands: {
    groups: {
      lenses: "Lenses",
      actions: "Actions",
    },
    goTo: (lens: string) => `Go to ${lens}`,
    showShortcuts: "Show the keyboard shortcuts",
  },
  shortcuts: {
    title: "Keyboard shortcuts",
    commands: "Commands",
    moving: "Lists and the finder",
    keys: [
      { keys: "J", what: "Next line or row" },
      { keys: "K", what: "Previous line or row" },
      { keys: "↓ ↑", what: "Next or previous, also in the finder" },
      { keys: "esc", what: "Close the pane, the finder or this sheet" },
      { keys: "⌘K", what: "Search or run a command" },
      { keys: "↵", what: "Run what the finder selected" },
    ],
  },
  finder: {
    box: "Search or run a command",
    boxKey: "⌘K",
    title: "Search or run a command",
    placeholder: "Go to a lens, find a task, or run a command",
    groups: {
      channels: "Channels",
      tasks: "Tasks",
      messages: "Messages",
      agents: "Agents",
    },
    channelMark: "#",
    channel: (channel: string) => `#${channel}`,
    handleMark: "@",
    writtenBy: (author: string, channel: string, createdAt: string) =>
      `${author} · #${channel} · ${dayOfTheMonth.format(new Date(createdAt))}`,
    noMatch: "No match",
    move: "↑↓ move",
    run: "↵ run",
    close: "esc close",
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
