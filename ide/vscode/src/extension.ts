import * as vscode from "vscode";
import * as client from "vscode-languageclient/node";
import * as path from "path";
import { SyntaxTreeProvider } from "./syntax-tree";

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

  const syntaxTreeProvider = new SyntaxTreeProvider(context, c);

  const disposeProvider = vscode.window.registerTreeDataProvider(
    "rhaiSyntaxTree",
    syntaxTreeProvider
  );

  syntaxTreeProvider.setEditor(vscode.window.activeTextEditor);

  context.subscriptions.push(
    disposeProvider,
    vscode.window.onDidChangeActiveTextEditor(editor => {
      if (!editor || editor.document.languageId !== "rhai") {
        syntaxTreeProvider.setEditor(undefined);
        return;
      }

      syntaxTreeProvider.setEditor(editor);
    }),
    vscode.workspace.onDidChangeTextDocument(() => {
      // Let the LSP parse the document.
      setTimeout(() => {
        syntaxTreeProvider.update();
      }, 100);
    })
  );

  c.registerProposedFeatures();

  context.subscriptions.push(output, c.start());
  await c.onReady();
}
