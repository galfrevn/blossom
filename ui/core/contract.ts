export type Person = { handle: string, name: string, };

export type OpenBoard = { "kind": "noneRunning" } | { "kind": "open", url: string, person: Person, };
