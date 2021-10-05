// @ts-ignore
import loadRhai from "../../../crates/lsp/Cargo.toml";

declare const window: any;

export interface SyntaxTree {
  kind: string;
  text_range: [number, number];
  children?: Array<SyntaxTree>;
  text?: string;
}

export interface Range {
  start: Position;
  end: Position;
}

export interface Position {
  line: number;
  character: number;
}

/**
 * Additional requests for the server that are not in the official LSP specification.
 */
export namespace Requests {
  /**
   * Sent from the client to the server.
   *
   * Get the syntax tree of the given document.
   */
  export namespace SyntaxTreeRequest {
    export interface SyntaxTreeParams {
      /**
       * URI of the document.
       */
      uri: string;
    }

    export type SyntaxTreeResponse = {
      /**
       * Syntax tree textual representation.
       */
      text: string;
      tree: SyntaxTree;
    } | null;

    export const METHOD = "rhai/syntaxTree";
  }

  /**
   * Sent from the client to the server.
   *
   * Convert 0-based byte text offsets to 1-based row:column representations
   * for the given document.
   */
  export namespace ConvertOffsets {
    export interface ConvertOffsetsParams {
      /**
       * URI of the document.
       */
      uri: string;
      ranges?: Array<[number, number]>;
      positions?: Array<number>;
    }

    export type ConvertOffsetsResponse = {
      ranges?: Array<Range>;
      positions?: Array<Position>;
    } | null;

    export const METHOD = "rhai/convertOffsets";
  }
}

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
