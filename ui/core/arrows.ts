const STEP_OF_EACH_ARROW = new Map([
  ["ArrowUp", -1],
  ["ArrowDown", 1],
]);

const STEP_OF_EACH_LETTER = new Map([
  ["k", -1],
  ["j", 1],
]);

function isPressedWithModifiers(event: KeyboardEvent): boolean {
  return event.metaKey || event.ctrlKey || event.altKey || event.shiftKey;
}

export function arrowStepOf(event: KeyboardEvent): number | undefined {
  if (isPressedWithModifiers(event)) {
    return undefined;
  }
  return STEP_OF_EACH_ARROW.get(event.key);
}

export function listStepOf(event: KeyboardEvent): number | undefined {
  if (isPressedWithModifiers(event)) {
    return undefined;
  }
  return (
    STEP_OF_EACH_ARROW.get(event.key) ?? STEP_OF_EACH_LETTER.get(event.key)
  );
}

export function comesFromTheList(event: KeyboardEvent, list: HTMLElement) {
  if (event.target === document.body) {
    return true;
  }
  return event.target instanceof Node && list.contains(event.target);
}
