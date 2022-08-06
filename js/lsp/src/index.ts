import loadRhai from "../../../crates/rhai-wasm/Cargo.toml";
import { convertEnv, Environment } from "@rhaiscript/core";

export interface RpcMessage {
  jsonrpc: "2.0";
  method?: string;
  id?: string | number;
  params?: any;
  result?: any;
  error?: any;
}

export interface LspInterface {
  /**
   * Handler for RPC messages set from the LSP server.
   */
  onMessage: (message: RpcMessage) => void;
}

export class RhaiLsp {
  private static rhai: any | undefined;
  private static initializing: boolean = false;

  private constructor(private lspInner: any) {
    if (!RhaiLsp.initializing) {
      throw new Error(
        `an instance of RhaiLsp can only be created by calling the "initialize" static method`
      );
    }
  }

  public static async initialize(
    env: Environment,
    lspInterface: LspInterface
  ): Promise<RhaiLsp> {
    if (typeof RhaiLsp.rhai === "undefined") {
      RhaiLsp.rhai = await loadRhai();
    }
    RhaiLsp.rhai.initialize();

    RhaiLsp.initializing = true;
    const t = new RhaiLsp(
      RhaiLsp.rhai.create_lsp(convertEnv(env), {
        js_on_message: lspInterface.onMessage,
      })
    );
    RhaiLsp.initializing = false;

    return t;
  }

  public send(message: RpcMessage) {
    this.lspInner.send(message);
  }

  public dispose() {
    this.lspInner.free();
  }
}
