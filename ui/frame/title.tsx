import type { ReactNode } from "react";

export function TitleBar({ children }: { children: ReactNode }) {
  return (
    <header
      data-tauri-drag-region={true}
      className="flex h-[38px] shrink-0 items-center justify-end border-line border-b pr-2.5 pl-4"
    >
      {children}
    </header>
  );
}
