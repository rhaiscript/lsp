// @ts-ignore
import loadRhai from "../../../crates/lsp/Cargo.toml";

declare const window: any;

export interface Handlers {
  /**
   * Handle a JSON RPC message from the server.
   * The message is an object, and not serialized JSON.
   */
  onMessage: (message: any) => void;
}

export class RhaiLsp {
  private static lsp: any | undefined;
  private static initializing: boolean = false;

  private constructor() {
    if (!RhaiLsp.initializing) {
      throw new Error(
        `an instance of RhaiLsp can only be created by calling the "initialize" static method`
      );
    }
  }

  /**
   * Initialize the language server.
   *
   * After initialization, the server will be ready to accept JSON RPC messages.
   * The only way to exit is exiting the process itself.
   *
   * @param {Handlers} handlers Handlers required for the server.
   */
  public static async initialize(handlers: Handlers) {
    if (typeof RhaiLsp.lsp === "undefined") {
      if (typeof global === "undefined") {
        window.__rhai__ = handlers;
      } else {
        (global as any).__rhai__ = handlers;
      }

      RhaiLsp.lsp = await loadRhai();
      RhaiLsp.lsp.initialize();
    }

    RhaiLsp.initializing = true;
    const t = new RhaiLsp();
    RhaiLsp.initializing = false;

    return t;
  }

  /**
   * Send a JSON RPC message to the server.
   * The message must be an object, and not serialized JSON.
   */
  public message(message: any) {
    RhaiLsp.lsp.message(message);
  }
}
