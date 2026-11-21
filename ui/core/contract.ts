export type Person = { handle: string, name: string, };

export type OpenBoard = { "kind": "noneRunning" } | { "kind": "open", url: string, person: Person, };

export type StopReason = "unreadableBoard" | "unwritableReplica";

export type ReplicaState = { "kind": "copying", stored: number, total: number | null, } | { "kind": "live", stored: number, } | { "kind": "reconnecting", stored: number, } | { "kind": "stopped", stored: number, reason: StopReason, };
