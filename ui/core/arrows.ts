const STEP_OF_EACH_ARROW = new Map([
  ["ArrowUp", -1],
  ["ArrowDown", 1],
]);

export function arrowStepOf(event: KeyboardEvent): number | undefined {
  if (event.metaKey || event.ctrlKey || event.altKey || event.shiftKey) {
    return undefined;
  }
  return STEP_OF_EACH_ARROW.get(event.key);
}

export function comesFromTheList(event: KeyboardEvent, list: HTMLElement) {
  if (event.target === document.body) {
    return true;
  }
  return event.target instanceof Node && list.contains(event.target);
}
