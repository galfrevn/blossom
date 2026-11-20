import { dictionary } from "./dictionary";

const COMMAND_THAT_STARTS_A_LOCAL_BOARD = "vb serve";

export function StartScreen() {
  const text = dictionary.start;
  return (
    <section className="m-auto flex max-w-[360px] flex-col items-start gap-3">
      <h1 className="font-pixel text-[22px] leading-none">{text.title}</h1>
      <p className="text-ink-2">{text.instruction}</p>
      <code className="rounded-[3px] border border-line bg-sunken px-2 py-1 font-mono text-[12px]">
        {COMMAND_THAT_STARTS_A_LOCAL_BOARD}
      </code>
      <p className="text-ink-3">{text.afterwards}</p>
    </section>
  );
}
