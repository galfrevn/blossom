import { dictionary } from "../dictionary";

function NarrowedTo({
  channel,
  onWiden,
}: {
  channel: string | null;
  onWiden: () => void;
}) {
  if (channel === null) {
    return null;
  }
  return (
    <span className="flex items-center gap-1.5 rounded-[5px] border border-line bg-raised py-0.5 pr-1 pl-2 font-mono text-[11px] text-ink-2">
      {dictionary.lenses.narrowedTo(channel)}
      <button
        type="button"
        aria-label={dictionary.lenses.widen}
        onClick={onWiden}
        className="rounded-[3px] px-1 text-ink-3 hover:bg-sunken hover:text-ink"
      >
        {dictionary.lenses.widenMark}
      </button>
    </span>
  );
}

export function LensHeader({
  title,
  description,
  channel,
  onWiden,
}: {
  title: string;
  description: string;
  channel: string | null;
  onWiden: () => void;
}) {
  return (
    <header className="flex flex-wrap items-end justify-between gap-x-[18px] gap-y-2.5 border-line border-b px-5 pt-[18px] pb-3.5">
      <div>
        <h1 className="font-pixel text-[30px] leading-none tracking-[0.005em]">
          {title}
        </h1>
        <p className="mt-1.5 max-w-[64ch] text-[12.5px] text-ink-2">
          {description}
        </p>
      </div>
      <NarrowedTo channel={channel} onWiden={onWiden} />
    </header>
  );
}
