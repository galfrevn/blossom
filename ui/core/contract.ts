export type Person = { handle: string, name: string, };

export type OpenBoard = { "kind": "noneRunning" } | { "kind": "open", url: string, person: Person, };

export type StopReason = "unreadableBoard" | "unwritableReplica";

export type ReplicaState = { "kind": "copying", stored: number, total: number | null, } | { "kind": "live", stored: number, } | { "kind": "reconnecting", stored: number, } | { "kind": "stopped", stored: number, reason: StopReason, };

export type FlowRow = { id: string, channel: string, author: string, onBehalfOf: string | null, createdAt: string, text: string, edited: boolean, deleted: boolean, firstOfItsMinute: boolean, fold: FlowFold | null, };

export type FlowFold = { count: number, agents: number, recent: Array<FlowRow>, };

export type FlowWindow = { total: number, start: number, rows: Array<FlowRow>, };

export type FlowChange = { revision: number, };
