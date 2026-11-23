import { useEffect, useLayoutEffect, useRef } from "react";
import type {
  FoundAgent,
  FoundMessage,
  FoundTask,
  TextPart,
} from "../core/contract";
import { dictionary } from "../dictionary";
import { looksOf } from "../tasks/states";
import {
  type FinderGroup,
  type FinderOption,
  type GroupName,
  groupsOf,
  keyOf,
} from "./options";

type ListElement = React.RefObject<HTMLDivElement | null>;

const GROUP_NAMES: Record<GroupName, string> = {
  lenses: dictionary.commands.groups.lenses,
  actions: dictionary.commands.groups.actions,
  channel: dictionary.finder.groups.channels,
  task: dictionary.finder.groups.tasks,
  message: dictionary.finder.groups.messages,
};

const LABEL = "min-w-0 flex-1 truncate text-ink-2";
const GROUP_HEADING =
  "px-2.5 pt-2 pb-1 font-medium font-mono text-[10px] text-ink-3 uppercase tracking-[0.08em]";
const ROW =
  "flex w-full items-center justify-between gap-3 rounded-[5px] px-2.5 py-1.5 text-[12.5px]";
const HINT =
  "max-w-[45%] shrink-0 truncate whitespace-nowrap font-mono text-[10.5px] text-ink-3";

function Marked({ parts }: { parts: TextPart[] }) {
  return parts.map((part, index) => {
    const key = `${index}:${part.text}`;
    if (part.marked) {
      return (
        <mark key={key} className="bg-transparent font-semibold text-ink">
          {part.text}
        </mark>
      );
    }
    return <span key={key}>{part.text}</span>;
  });
}

function TaskHint({ found }: { found: FoundTask }) {
  const looks = looksOf(found.state);
  return (
    <span className={HINT}>
      <span className={`uppercase tracking-[0.06em] ${looks.ink}`}>
        {looks.name}
      </span>
      {` · ${dictionary.finder.channel(found.channel)}`}
    </span>
  );
}

function MessageHint({ found }: { found: FoundMessage }) {
  return (
    <span className={HINT}>
      {dictionary.finder.writtenBy(
        found.author,
        found.channel,
        found.createdAt,
      )}
    </span>
  );
}

function OptionContent({ option }: { option: FinderOption }) {
  if (option.kind === "command") {
    return (
      <>
        <span className={LABEL}>
          <Marked parts={option.marked} />
        </span>
        <kbd className={HINT}>{option.command.shortcut}</kbd>
      </>
    );
  }
  if (option.kind === "channel") {
    return (
      <span className={LABEL}>
        {dictionary.finder.channelMark}
        <Marked parts={option.found.markedPath} />
      </span>
    );
  }
  if (option.kind === "task") {
    return (
      <>
        <span className={LABEL}>
          <Marked parts={option.found.snippet} />
        </span>
        <TaskHint found={option.found} />
      </>
    );
  }
  return (
    <>
      <span className={LABEL}>
        <Marked parts={option.found.snippet} />
      </span>
      <MessageHint found={option.found} />
    </>
  );
}

function Group({
  group,
  listId,
  selectedIndex,
}: {
  group: FinderGroup;
  listId: string;
  selectedIndex: number;
}) {
  const name = GROUP_NAMES[group.name];
  return (
    <fieldset aria-label={name} className="min-w-0">
      <div aria-hidden={true} className={GROUP_HEADING}>
        {name}
      </div>
      {group.options.map((option, position) => {
        const index = group.firstIndex + position;
        return (
          <div
            key={keyOf(option)}
            id={`${listId}-${index}`}
            role="option"
            tabIndex={-1}
            aria-selected={index === selectedIndex}
            data-option={index}
            className={`${ROW} aria-selected:bg-accent-wash`}
          >
            <OptionContent option={option} />
          </div>
        );
      })}
    </fieldset>
  );
}

function AgentsFound({ agents }: { agents: FoundAgent[] }) {
  if (agents.length === 0) {
    return null;
  }
  const name = dictionary.finder.groups.agents;
  return (
    <fieldset aria-label={name} className="min-w-0">
      <div aria-hidden={true} className={GROUP_HEADING}>
        {name}
      </div>
      {agents.map((agent) => (
        <div
          key={agent.id}
          role="option"
          tabIndex={-1}
          aria-selected={false}
          aria-disabled={true}
          className={ROW}
        >
          <span className={LABEL}>
            <Marked parts={agent.markedName} />
          </span>
          <span className={HINT}>
            {dictionary.finder.handleMark}
            <Marked parts={agent.markedHandle} />
          </span>
        </div>
      ))}
    </fieldset>
  );
}

function optionClicked(
  event: MouseEvent,
  options: FinderOption[],
): FinderOption | undefined {
  if (!(event.target instanceof Element)) {
    return undefined;
  }
  const clicked = event.target.closest<HTMLElement>("[data-option]");
  if (clicked === null) {
    return undefined;
  }
  return options[Number(clicked.dataset.option)];
}

function useOpeningByClicks(
  list: ListElement,
  options: FinderOption[],
  onOpen: (option: FinderOption) => void,
) {
  useEffect(() => {
    const element = list.current;
    if (element === null) {
      return;
    }
    const keepTheKeyboardInTheBox = (event: MouseEvent) => {
      event.preventDefault();
    };
    const openTheClickedOption = (event: MouseEvent) => {
      const option = optionClicked(event, options);
      if (option !== undefined) {
        onOpen(option);
      }
    };
    element.addEventListener("mousedown", keepTheKeyboardInTheBox);
    element.addEventListener("click", openTheClickedOption);
    return () => {
      element.removeEventListener("mousedown", keepTheKeyboardInTheBox);
      element.removeEventListener("click", openTheClickedOption);
    };
  }, [list, options, onOpen]);
}

function useSelectionInView(
  list: ListElement,
  options: FinderOption[],
  selectedIndex: number,
) {
  useLayoutEffect(() => {
    if (options.length === 0) {
      return;
    }
    const selected = list.current?.querySelector(
      `[data-option="${selectedIndex}"]`,
    );
    selected?.scrollIntoView({ block: "nearest" });
  }, [list, options, selectedIndex]);
}

export function FinderResults({
  listId,
  options,
  agents,
  selectedIndex,
  onOpen,
}: {
  listId: string;
  options: FinderOption[];
  agents: FoundAgent[];
  selectedIndex: number;
  onOpen: (option: FinderOption) => void;
}) {
  const list = useRef<HTMLDivElement>(null);
  useOpeningByClicks(list, options, onOpen);
  useSelectionInView(list, options, selectedIndex);
  return (
    <div
      ref={list}
      id={listId}
      role="listbox"
      aria-label={dictionary.finder.title}
      className="max-h-[340px] overflow-auto p-1"
    >
      {options.length + agents.length === 0 && (
        <div className={GROUP_HEADING}>{dictionary.finder.noMatch}</div>
      )}
      {groupsOf(options).map((group) => (
        <Group
          key={`${group.name}:${group.firstIndex}`}
          group={group}
          listId={listId}
          selectedIndex={selectedIndex}
        />
      ))}
      <AgentsFound agents={agents} />
    </div>
  );
}
