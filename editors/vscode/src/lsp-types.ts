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