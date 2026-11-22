import type { Core } from "./client";
import type {
  Agent,
  FlowChange,
  FlowRow,
  OpenBoard,
  ReplicaState,
  SearchResults,
  TaskChange,
  TaskCount,
  TaskNarrowing,
  TaskRow,
  Thread,
} from "./contract";

function createFakeSubscription<Value>() {
  const listeners = new Set<(value: Value) => void>();
  let latestValue: Value | undefined;
  return {
    subscribe: (listener: (value: Value) => void) => {
      listeners.add(listener);
      if (latestValue !== undefined) {
        listener(latestValue);
      }
      return () => {
        listeners.delete(listener);
      };
    },
    push: (value: Value) => {
      latestValue = value;
      for (const listener of listeners) {
        listener(value);
      }
    },
  };
}

function isWithin(channel: string, narrowedTo: string | null): boolean {
  if (narrowedTo === null) {
    return true;
  }
  return channel === narrowedTo || channel.startsWith(`${narrowedTo}/`);
}

function comesAfter(row: FlowRow, createdAt: string, id: string): boolean {
  if (row.createdAt !== createdAt) {
    return row.createdAt > createdAt;
  }
  return row.id > id;
}

function threadHolding(threads: Thread[], messageId: string): Thread | null {
  const holding = threads.find((thread) =>
    thread.messages.some((message) => message.id === messageId),
  );
  return holding ?? null;
}

function noSearchResults(): SearchResults {
  return { messages: [], tasks: [], channels: [], agents: [] };
}

function isNarrowedTo(row: TaskRow, narrowing: TaskNarrowing): boolean {
  const title = narrowing.title.trim().toLowerCase();
  const inTheChannel = isWithin(row.channel, narrowing.channel);
  const inTheState = narrowing.state === null || row.state === narrowing.state;
  const titled = row.title.toLowerCase().includes(title);
  return inTheChannel && inTheState && titled;
}

function countsOfEachState(rows: TaskRow[]): TaskCount[] {
  const counts: TaskCount[] = [];
  for (const row of rows) {
    const counted = counts.find((count) => count.state === row.state);
    if (counted === undefined) {
      counts.push({ state: row.state, count: 1 });
    } else {
      counted.count += 1;
    }
  }
  return counts;
}

function createFakeTasks() {
  const tasks = createFakeSubscription<TaskChange>();
  let taskRows: TaskRow[] = [];
  let taskRevision = 0;
  return {
    subscribeToTasks: tasks.subscribe,
    readTasks: (narrowing: TaskNarrowing, start: number, count: number) => {
      const narrowed = taskRows.filter((row) => isNarrowedTo(row, narrowing));
      return Promise.resolve({
        total: narrowed.length,
        start,
        rows: narrowed.slice(start, start + count),
      });
    },
    readTaskCounts: (channel: string | null) => {
      const inTheChannel = taskRows.filter((row) =>
        isWithin(row.channel, channel),
      );
      return Promise.resolve(countsOfEachState(inTheChannel));
    },
    pushTasks: (rows: TaskRow[]) => {
      taskRows = rows;
      taskRevision += 1;
      tasks.push({ revision: taskRevision });
    },
  };
}

function createFakeLinks() {
  const linksOpened: string[] = [];
  return {
    openLink: (link: string) => {
      linksOpened.push(link);
      return Promise.resolve();
    },
    linksOpened: () => [...linksOpened],
  };
}

function createFakeFlow() {
  const flow = createFakeSubscription<FlowChange>();
  let flowRows: FlowRow[] = [];
  let threads: Thread[] = [];
  let flowRevision = 0;
  const announceTheChange = () => {
    flowRevision += 1;
    flow.push({ revision: flowRevision });
  };
  return {
    subscribeToFlow: flow.subscribe,
    readFlow: (channel: string | null, start: number, count: number) => {
      const narrowed = flowRows.filter((row) => isWithin(row.channel, channel));
      return Promise.resolve({
        total: narrowed.length,
        start,
        rows: narrowed.slice(start, start + count),
      });
    },
    countFlowAfter: (channel: string | null, createdAt: string, id: string) => {
      const after = flowRows.filter(
        (row) =>
          isWithin(row.channel, channel) && comesAfter(row, createdAt, id),
      );
      return Promise.resolve(after.length);
    },
    readThread: (messageId: string) =>
      Promise.resolve(threadHolding(threads, messageId)),
    pushFlow: (rows: FlowRow[]) => {
      flowRows = rows;
      announceTheChange();
    },
    pushThreads: (newThreads: Thread[]) => {
      threads = newThreads;
      announceTheChange();
    },
  };
}

export type FakeCore = {
  core: Core;
  pushOpenBoard: (openBoard: OpenBoard) => void;
  pushReplicaState: (replicaState: ReplicaState) => void;
  pushChannels: (channels: string[]) => void;
  pushFlow: (rows: FlowRow[]) => void;
  pushThreads: (threads: Thread[]) => void;
  pushFleet: (agents: Agent[]) => void;
  answerSearch: (query: string, results: SearchResults) => void;
  linksOpened: () => string[];
  pushTasks: (rows: TaskRow[]) => void;
};

export function createFakeCore(): FakeCore {
  const openBoard = createFakeSubscription<OpenBoard>();
  const replica = createFakeSubscription<ReplicaState>();
  const channels = createFakeSubscription<string[]>();
  const fleet = createFakeSubscription<Agent[]>();
  const { pushFlow, pushThreads, ...flow } = createFakeFlow();
  const { pushTasks, ...tasks } = createFakeTasks();
  const { linksOpened, ...links } = createFakeLinks();
  const searchAnswers = new Map<string, SearchResults>();
  return {
    core: {
      subscribeToOpenBoard: openBoard.subscribe,
      subscribeToReplica: replica.subscribe,
      subscribeToChannels: channels.subscribe,
      subscribeToFleet: fleet.subscribe,
      search: (query) =>
        Promise.resolve(searchAnswers.get(query) ?? noSearchResults()),
      ...flow,
      ...links,
      ...tasks,
    },
    pushOpenBoard: openBoard.push,
    pushReplicaState: replica.push,
    pushChannels: channels.push,
    pushFlow,
    pushThreads,
    pushFleet: fleet.push,
    answerSearch: (query, results) => {
      searchAnswers.set(query, results);
    },
    linksOpened,
    pushTasks,
  };
}
