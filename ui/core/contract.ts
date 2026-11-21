export type Person = { handle: string, name: string, };

export type OpenBoard = { "kind": "noneRunning" } | { "kind": "open", url: string, person: Person, };

export type ReplicaState = { "kind": "copying", stored: number, total: number | null, } | { "kind": "copied", stored: number, };
