import * as vscode from "vscode";
import { SyntaxTreeProvider } from "./syntax-tree";

import { createClient } from "./client";
import { getOutput } from "./util";
import { registerCommands } from "./commands";

export async function activate(context: vscode.ExtensionContext) {
  try {
    let c = await createClient(context);

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

    context.subscriptions.push(getOutput(), c.start());

    registerCommands(context, c);
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
  } catch (e) {
    getOutput().appendLine(e.message);
  }
}
