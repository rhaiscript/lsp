import * as vscode from "vscode";
import * as client from "vscode-languageclient/node";
import * as path from "path";
import { SyntaxTreeProvider } from "./syntax-tree";
import {
  Middleware,
  ProvideCodeLensesSignature,
  ResolveCodeLensSignature,
} from "vscode-languageclient/node";

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
    middleware: new ClientMiddleware(),
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
  vscode.commands.executeCommand("setContext", "rhai.extensionActive", true);
  context.subscriptions.push({
    dispose: () => {
      vscode.commands.executeCommand(
        "setContext",
        "rhai.extensionActive",
        false
      );
    },
  });
}

class ClientMiddleware implements Middleware {
  // Mostly taken from a PowerShell extension (https://github.com/microsoft/vscode-languageserver-node/issues/495#issuecomment-519109203)
  public provideCodeLenses(
    doc: vscode.TextDocument,
    token: vscode.CancellationToken,
    next: ProvideCodeLensesSignature
  ): vscode.ProviderResult<Array<vscode.CodeLens>> {
    const resolvedCodeLenses = next(doc, token);
    const fixCodeLens = (codeLensToFix: vscode.CodeLens): vscode.CodeLens => {
      if (codeLensToFix.command?.command === "editor.action.showReferences") {
        const oldArgs = codeLensToFix.command.arguments;

        // Our JSON objects don't get handled correctly by
        // VS Code's built in editor.action.showReferences
        // command so we need to convert them into the
        // appropriate types to send them as command
        // arguments.

        codeLensToFix.command.arguments = [
          vscode.Uri.parse(oldArgs[0]),
          new vscode.Position(oldArgs[1].line, oldArgs[1].character),
          oldArgs[2].map(position => {
            return new vscode.Location(
              vscode.Uri.parse(position.uri),
              new vscode.Range(
                position.range.start.line,
                position.range.start.character,
                position.range.end.line,
                position.range.end.character
              )
            );
          }),
        ];
      }

      return codeLensToFix;
    };

    if (isThenable<Array<vscode.CodeLens>>(resolvedCodeLenses)) {
      return resolvedCodeLenses.then(r => r.map(fixCodeLens));
    } else if (is<Array<vscode.CodeLens>>(resolvedCodeLenses)) {
      return resolvedCodeLenses.map(fixCodeLens);
    }

    return resolvedCodeLenses;
  }
}

function isThenable<T>(obj: any): obj is Thenable<T> {
  return (
    typeof obj !== "undefined" &&
    obj !== null &&
    typeof obj.then !== "undefined"
  );
}

function is<T>(obj: any): obj is T {
  return (
    typeof obj !== "undefined" &&
    obj !== null &&
    typeof obj.then === "undefined"
  );
}
