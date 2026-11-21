import { useCallback, useState } from "react";
import markUrl from "../../desktop/icons/large.svg";
import { dictionary } from "../dictionary";

function copyLabelOf(copied: boolean): string {
  if (copied) {
    return dictionary.emptyBoard.copied;
  }
  return dictionary.emptyBoard.copy;
}

export function EmptyBoard({ boardUrl }: { boardUrl: string }) {
  const text = dictionary.emptyBoard;
  const prompt = text.prompt(boardUrl);
  const [copied, setCopied] = useState(false);
  const copy = useCallback(() => {
    navigator.clipboard.writeText(prompt).then(
      () => setCopied(true),
      (error: unknown) => {
        console.error("blossom: the prompt could not be copied", error);
      },
    );
  }, [prompt]);
  return (
    <section className="grid max-w-[560px] justify-items-start gap-3.5 px-5 pt-9 pb-8">
      <img
        src={markUrl}
        alt=""
        width={64}
        height={64}
        className="[image-rendering:pixelated]"
      />
      <h2 className="font-pixel text-[22px] leading-[1.1]">{text.title}</h2>
      <p className="max-w-[56ch] text-ink-2">{text.explanation}</p>
      <div className="w-full rounded-[6px] border border-line bg-raised">
        <pre className="m-0 cursor-text select-text whitespace-pre-wrap px-3.5 py-3 font-mono text-[12px] text-ink leading-[1.55] [overflow-wrap:anywhere]">
          {prompt}
        </pre>
        <div className="flex items-center justify-between gap-2.5 border-line border-t py-1.5 pr-2 pl-3.5 font-mono text-[10.5px] text-ink-3">
          <span>{text.where}</span>
          <button
            type="button"
            onClick={copy}
            className="rounded-[5px] border border-line bg-paper px-2.5 py-1 text-ink-2 hover:text-ink"
          >
            {copyLabelOf(copied)}
          </button>
        </div>
      </div>
    </section>
  );
}
