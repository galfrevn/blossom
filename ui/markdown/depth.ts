export const DEEPEST_NESTING = 8;

export function isPastTheDeepestNesting(depth: number): boolean {
  return depth > DEEPEST_NESTING;
}
