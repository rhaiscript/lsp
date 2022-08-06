import * as vscode from "vscode";
import * as client from "vscode-languageclient/node";
import { Lsp } from "@rhaiscript/core";

export class SyntaxTreeProvider
  implements vscode.TreeDataProvider<SyntaxTreeItem>
{
  private readonly syntaxDecorationType =
    vscode.window.createTextEditorDecorationType({
      backgroundColor: new vscode.ThemeColor("editor.wordHighlightBackground"),
    });

  private editor?: vscode.TextEditor;

  private _onDidChangeTreeData: vscode.EventEmitter<
    SyntaxTreeItem | undefined | null | void
  > = new vscode.EventEmitter<SyntaxTreeItem | undefined | null | void>();
  readonly onDidChangeTreeData: vscode.Event<
    SyntaxTreeItem | undefined | null | void
  > = this._onDidChangeTreeData.event;

  setEditor(editor?: vscode.TextEditor): void {
    this.editor = editor;
    this.update();
  }

  update(): void {
    this.editor?.setDecorations(this.syntaxDecorationType, []);
    this._onDidChangeTreeData.fire();
  }

  constructor(
    context: vscode.ExtensionContext,
    private readonly client: client.LanguageClient
  ) {
    context.subscriptions.push(
      vscode.commands.registerCommand(
        "highlightRhaiSyntax",
        async (textRange: [number, number]) => {
          if (!this.editor) {
            return;
          }

          const params: Lsp.Client.RequestParams<"rhai/convertOffsets"> = {
            uri: this.editor.document.uri.toString(),
            ranges: [textRange],
          };

          const res = await this.client.sendRequest<
            Lsp.Client.RequestResponse<"rhai/convertOffsets">
          >("rhai/convertOffsets", params);

          const lspRange = res.ranges?.[0];

          if (lspRange) {
            const range = new vscode.Range(
              lspRange.start.line,
              lspRange.start.character,
              lspRange.end.line,
              lspRange.end.character
            );

            this.editor.setDecorations(this.syntaxDecorationType, [range]);
            this.editor.revealRange(range);
          }
        }
      )
    );
  }

  getTreeItem(element: SyntaxTreeItem): vscode.TreeItem {
    return element;
  }

  async getChildren(element?: SyntaxTreeItem): Promise<SyntaxTreeItem[]> {
    if (element) {
      return (
        element.syntax.children
          ?.filter(c => c.kind !== "WHITESPACE")
          .map(syntax => new SyntaxTreeItem(syntax)) ?? []
      );
    }

    if (!this.editor) {
      return [];
    }

    await this.client.onReady();

    const params: Lsp.Client.RequestParams<"rhai/syntaxTree"> = {
      uri: this.editor.document.uri.toString(),
    };

    const res = await this.client.sendRequest<
      Lsp.Client.RequestResponse<"rhai/syntaxTree">
    >("rhai/syntaxTree", params);

    if (!res) {
      // Try again, the document is probably not yet parsed.
      //
      // safety: Endlessly looping here is fine, at this point
      //  the editor is guaranteed to be "rhai", and we will eventually get a tree.
      setTimeout(() => {
        this.update();
      }, 300);
      return [];
    }

    return [new SyntaxTreeItem(res.tree)];
  }
}

class SyntaxTreeItem extends vscode.TreeItem {
  constructor(public readonly syntax: Lsp.SyntaxTree) {
    super(
      syntax.kind,
      (syntax.children?.length ?? 0) > 0
        ? ["RHAI", "ITEM", "STMT", "EXPR"].includes(syntax.kind)
          ? vscode.TreeItemCollapsibleState.Expanded
          : vscode.TreeItemCollapsibleState.Collapsed
        : vscode.TreeItemCollapsibleState.None
    );
    this.description = `${syntax.text_range[0]}..${syntax.text_range[1]}`;
    this.command = {
      command: "highlightRhaiSyntax",
      title: "Highlight Rhai Syntax",
      arguments: [this.syntax.text_range],
    };
  }
}
