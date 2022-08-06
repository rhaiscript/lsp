import * as vscode from "vscode";
import * as node from "vscode-languageclient/node";
import { Middleware, ProvideCodeLensesSignature } from "vscode-languageclient/node";
import which from "which";
import { getOutput } from "./util";

export async function createClient(context: vscode.ExtensionContext) {
  const out = getOutput();

  const bundled = !!vscode.workspace
    .getConfiguration()
    .get("rhai.executable.bundled");

  let serverOpts: node.ServerOptions;
  if (bundled) {
    const rhaiPath = vscode.Uri.joinPath(
      context.extensionUri,
      "dist/server.js"
    ).fsPath;

    const run: node.NodeModule = {
      module: rhaiPath,
      transport: node.TransportKind.ipc,
      options: {
        env:
          vscode.workspace
            .getConfiguration()
            .get("rhai.executable.environment") ?? undefined,
      },
    };

    serverOpts = {
      run,
      debug: run,
    };
  } else {
    const rhaiPath =
      vscode.workspace.getConfiguration().get("rhai.executable.path") ??
      which.sync("rhai", { nothrow: true });

    if (typeof rhaiPath !== "string") {
      out.appendLine("failed to locate rhai executable");
      throw new Error("failed to locate rhai executable");
    }

    let extraArgs = vscode.workspace
      .getConfiguration()
      .get("rhai.executable.extraArgs");

    if (!Array.isArray(extraArgs)) {
      extraArgs = [];
    }

    const args: string[] = (extraArgs as any[]).filter(
      a => typeof a === "string"
    );

    const run: node.Executable = {
      command: rhaiPath,
      args: ["lsp", "stdio", ...args],
      options: {
        env:
          vscode.workspace
            .getConfiguration()
            .get("rhai.executable.environment") ?? undefined,
      },
    };

    serverOpts = {
      run,
      debug: run,
    };
  }

  return new node.LanguageClient(
    "rhaiLsp",
    "Rhai Language Server",
    serverOpts,
    await clientOpts(context)
  );
}

async function clientOpts(context: vscode.ExtensionContext): Promise<any> {
  await vscode.workspace.fs.createDirectory(context.globalStorageUri);

  return {
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
