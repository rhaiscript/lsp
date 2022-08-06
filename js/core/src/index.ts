export * from "./environment";
export * as Lsp from "./lsp";
export * from "./config";

/**
 * Byte range within a document.
 */
export interface Range {
  /**
   * Start byte index.
   */
  start: number;
  /**
   * Exclusive end index.
   */
  end: number;
}

