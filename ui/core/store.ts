import { useSyncExternalStore } from "react";

export type Store<Value> = {
  subscribe: (onChange: () => void) => () => void;
  getSnapshot: () => Value | undefined;
};

export function createStore<Value>(
  subscribeToCore: (listener: (value: Value) => void) => () => void,
): Store<Value> {
  let currentValue: Value | undefined;
  const changeListeners = new Set<() => void>();
  subscribeToCore((value) => {
    currentValue = value;
    for (const onChange of changeListeners) {
      onChange();
    }
  });
  return {
    subscribe: (onChange) => {
      changeListeners.add(onChange);
      return () => {
        changeListeners.delete(onChange);
      };
    },
    getSnapshot: () => currentValue,
  };
}

export function useStore<Value>(store: Store<Value>): Value | undefined {
  return useSyncExternalStore(store.subscribe, store.getSnapshot);
}
