export type Person = { handle: string, name: string, };

export type OpenBoard = { "kind": "noneRunning" } | { "kind": "open", url: string, person: Person, };

export type StopReason = "unreadableBoard" | "unwritableReplica";

export type ReplicaState = { "kind": "copying", stored: number, total: number | null, } | { "kind": "live", stored: number, } | { "kind": "reconnecting", stored: number, } | { "kind": "stopped", stored: number, reason: StopReason, };

export type MentionedActor = { id: string, name: string, };

export type FlowRow = { id: string, channel: string, author: string, onBehalfOf: string | null, createdAt: string, text: string, mentions: Array<MentionedActor>, edited: boolean, deleted: boolean, firstOfItsMinute: boolean, fold: FlowFold | null, };

export type FlowFold = { count: number, agents: number, recent: Array<FlowRow>, };

export type FlowWindow = { total: number, start: number, rows: Array<FlowRow>, };

export type FlowChange = { revision: number, };

export type ThreadMessage = { id: string, author: string, onBehalfOf: string | null, createdAt: string, text: string, mentions: Array<MentionedActor>, edited: boolean, deleted: boolean, };

export type Thread = { id: string, channel: string, messages: Array<ThreadMessage>, };

export type HeldTask = { task: string, until: string, };

export type Agent = { id: string, name: string, onBehalfOf: string | null, holds: HeldTask | null, lapsed: boolean, failedRecently: boolean, channel: string | null, };

export type TextPart = { text: string, marked: boolean, };

export type FoundMessage = { id: string, channel: string, author: string, createdAt: string, snippet: Array<TextPart>, };

export type FoundTask = { id: string, channel: string, state: string, snippet: Array<TextPart>, };

export type FoundChannel = { path: string, markedPath: Array<TextPart>, };

export type FoundAgent = { id: string, markedName: Array<TextPart>, markedHandle: Array<TextPart>, };

export type SearchResults = { messages: Array<FoundMessage>, tasks: Array<FoundTask>, channels: Array<FoundChannel>, agents: Array<FoundAgent>, };

export type TaskNarrowing = { channel: string | null, state: string | null, title: string, };

export type TaskRow = { id: string, state: string, title: string, channel: string, owner: string, holder: string | null, lease: Lease | null, changedAt: string, };

export type Lease = { since: string | null, until: string, };

export type TaskWindow = { total: number, start: number, rows: Array<TaskRow>, };

export type TaskCount = { state: string, count: number, };

export type TaskChange = { revision: number, };
