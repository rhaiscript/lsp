export interface Range {
  start: Position;
  end: Position;
}

export interface Position {
  line: number;
  character: number;
}

export interface SyntaxTree {
  kind: string;
  text_range: [number, number];
  children?: Array<SyntaxTree>;
  text?: string;
}

export namespace Server {
  interface ServerNotifications {}

  export type NotificationMethod = keyof ServerNotifications;

  export type NotificationParams<T extends keyof ServerNotifications> =
    ServerNotifications[T] extends NotificationDescription
      ? ServerNotifications[T]["params"]
      : never;
}

export namespace Client {
  interface ClientNotifications {}

  interface ClientRequests {
    "rhai/hirDump": {
      params: {
        workspaceUri?: string;
      };
      response:
        | {
            hir: string;
          }
        | null
        | undefined;
    };
    "rhai/syntaxTree": {
      params: {
        /**
         * URI of the document.
         */
        uri: string;
      };
      response: {
        /**
         * Syntax tree textual representation.
         */
        text: string;
        tree: SyntaxTree;
      } | null;
    };
    "rhai/convertOffsets": {
      params: {
        /**
         * URI of the document.
         */
        uri: string;
        ranges?: Array<[number, number]>;
        positions?: Array<number>;
      };
      response: {
        ranges?: Array<Range>;
        positions?: Array<Position>;
      } | null;
    };
  }

  export type NotificationMethod = keyof ClientNotifications;

  export type NotificationParams<T extends keyof ClientNotifications> =
    ClientNotifications[T] extends NotificationDescription
      ? ClientNotifications[T]["params"]
      : never;

  export type RequestMethod = keyof ClientRequests;

  export type RequestParams<T extends keyof ClientRequests> =
    ClientRequests[T] extends RequestDescription
      ? ClientRequests[T]["params"]
      : never;

  export type RequestResponse<T extends keyof ClientRequests> =
    ClientRequests[T] extends RequestDescription
      ? ClientRequests[T]["response"]
      : never;
}

interface NotificationDescription {
  readonly params: any;
}

interface RequestDescription {
  readonly params: any;
  readonly response: any;
}

export type AssociationRule =
  | { glob: string }
  | { regex: string }
  | { url: string };

export interface SchemaInfo {
  url: string;
  meta: any;
}
