import * as vscode from "vscode";
import * as client from "vscode-languageclient/node";
import type { Lsp } from "@rhaiscript/core";

export function registerCommands(
  context: vscode.ExtensionContext,
  client: client.BaseLanguageClient
) {
  context.subscriptions.push(
    vscode.commands.registerCommand("rhai.showHirDump", async () => {
      const editor = vscode.window.activeTextEditor;

      if (!editor) {
        return;
      }

      const wsUri = vscode.workspace.getWorkspaceFolder(
        editor.document.uri
      ).uri;

      const s: Lsp.Client.RequestMethod = "rhai/hirDump";
      const params: Lsp.Client.RequestParams<"rhai/hirDump"> = {
        workspaceUri: wsUri.toString(),
      };
      const res = await client.sendRequest<Lsp.Client.RequestResponse<"rhai/hirDump">>(s, params);

      if (res) {
        const doc = await vscode.workspace.openTextDocument({content: res.hir});
        vscode.window.showTextDocument(doc);
      }
    })
  );
}
