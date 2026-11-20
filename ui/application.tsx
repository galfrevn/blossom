import { TitleBar } from "./frame/title";

export function Application() {
  return (
    <div className="flex h-screen flex-col bg-paper text-ink">
      <TitleBar />
      <main className="flex-1" />
    </div>
  );
}
