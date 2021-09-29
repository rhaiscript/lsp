import * as vscode from "vscode";
import * as client from "vscode-languageclient/node";
import * as path from "path";

let output: vscode.OutputChannel;

export function getOutput(): vscode.OutputChannel {
  return output;
}

export async function activate(context: vscode.ExtensionContext) {
  output = vscode.window.createOutputChannel("Rhai");

  let p = context.asAbsolutePath(path.join("dist", "server.js"));

  let serverOpts: client.ServerOptions = {
    run: { module: p, transport: client.TransportKind.ipc },
    debug: { module: p, transport: client.TransportKind.ipc },
  };

  let clientOpts: client.LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "rhai" }],

    initializationOptions: {
      configuration: vscode.workspace.getConfiguration().get("rhai"),
    },

    synchronize: {
      configurationSection: "rhai",
      fileEvents: [vscode.workspace.createFileSystemWatcher("**/*.rhai")],
    },
  };

  let c = new client.LanguageClient("rhai", "Rhai LSP", serverOpts, clientOpts);

  c.registerProposedFeatures();

  context.subscriptions.push(output, c.start());
  await c.onReady();
}
