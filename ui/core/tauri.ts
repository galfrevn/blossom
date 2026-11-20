import { Channel, invoke } from "@tauri-apps/api/core";
import type { Core } from "./client";
import type { OpenBoard } from "./contract";

export function createTauriCore(): Core {
  return {
    subscribeToOpenBoard(listener) {
      const channel = new Channel<OpenBoard>();
      let subscribed = true;
      channel.onmessage = (openBoard) => {
        if (subscribed) {
          listener(openBoard);
        }
      };
      invoke("subscribe_to_open_board", { channel }).catch((error: unknown) => {
        console.error("blossom: the core refused the subscription", error);
      });
      return () => {
        subscribed = false;
      };
    },
  };
}
