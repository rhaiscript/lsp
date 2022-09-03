use std::io::{self, Write};

use rhai_rowan::{
    ast::AstNode,
    syntax::{SyntaxElement, SyntaxKind::*},
};

use crate::{algorithm::Formatter, util::ScopedStatic};

impl<S: Write> Formatter<S> {
    #[allow(clippy::missing_panics_doc)]
    pub fn format(mut self, element: impl Into<SyntaxElement>) -> io::Result<()> {
        self.fmt_element(element.into())?;
        self.eof()
    }

    fn fmt_element(&mut self, element: SyntaxElement) -> io::Result<()> {
        let node = match element {
            rowan::NodeOrToken::Node(n) => n,
            rowan::NodeOrToken::Token(t)
                if matches!(
                    t.kind(),
                    COMMENT_BLOCK | COMMENT_LINE | COMMENT_BLOCK_DOC | COMMENT_LINE_DOC | SHEBANG
                ) =>
            {
                self.word(t.static_text())?;
                self.hardbreak();
                return Ok(());
            }
            rowan::NodeOrToken::Token(_) => return Ok(()),
        };

        match node.kind() {
            RHAI => self.fmt_rhai(AstNode::cast(node).unwrap())?,
            STMT => self.fmt_stmt(AstNode::cast(node).unwrap())?,
            ITEM => self.fmt_item(AstNode::cast(node).unwrap())?,
            DOC => self.fmt_doc(AstNode::cast(node).unwrap())?,
            EXPR => self.fmt_expr(AstNode::cast(node).unwrap())?,
            EXPR_IDENT => self.fmt_expr_ident(AstNode::cast(node).unwrap())?,
            EXPR_PATH => self.fmt_expr_path(AstNode::cast(node).unwrap())?,
            EXPR_LIT => self.fmt_expr_lit(AstNode::cast(node).unwrap())?,
            EXPR_LET => self.fmt_expr_let(AstNode::cast(node).unwrap())?,
            EXPR_CONST => self.fmt_expr_const(AstNode::cast(node).unwrap())?,
            EXPR_BLOCK => self.fmt_expr_block(AstNode::cast(node).unwrap(), false)?,
            EXPR_UNARY => self.fmt_expr_unary(AstNode::cast(node).unwrap())?,
            EXPR_BINARY => self.fmt_expr_binary(AstNode::cast(node).unwrap())?,
            EXPR_PAREN => self.fmt_expr_paren(AstNode::cast(node).unwrap())?,
            EXPR_ARRAY => self.fmt_expr_array(AstNode::cast(node).unwrap())?,
            EXPR_INDEX => self.fmt_expr_index(AstNode::cast(node).unwrap())?,
            EXPR_OBJECT => self.fmt_expr_object(AstNode::cast(node).unwrap())?,
            EXPR_CALL => self.fmt_expr_call(AstNode::cast(node).unwrap())?,
            EXPR_CLOSURE => self.fmt_expr_closure(AstNode::cast(node).unwrap())?,
            EXPR_IF => self.fmt_expr_if(AstNode::cast(node).unwrap())?,
            EXPR_LOOP => self.fmt_expr_loop(AstNode::cast(node).unwrap())?,
            EXPR_FOR => self.fmt_expr_for(AstNode::cast(node).unwrap())?,
            EXPR_WHILE => self.fmt_expr_while(AstNode::cast(node).unwrap())?,
            EXPR_BREAK => self.fmt_expr_break(AstNode::cast(node).unwrap())?,
            EXPR_CONTINUE => self.fmt_expr_continue(AstNode::cast(node).unwrap())?,
            EXPR_SWITCH => self.fmt_expr_switch(AstNode::cast(node).unwrap())?,
            EXPR_RETURN => self.fmt_expr_return(AstNode::cast(node).unwrap())?,
            EXPR_FN => self.fmt_expr_fn(AstNode::cast(node).unwrap())?,
            EXPR_EXPORT => self.fmt_expr_export(AstNode::cast(node).unwrap())?,
            EXPR_IMPORT => self.fmt_expr_import(AstNode::cast(node).unwrap())?,
            EXPR_TRY => self.fmt_expr_try(AstNode::cast(node).unwrap())?,
            EXPR_THROW => self.fmt_expr_throw(AstNode::cast(node).unwrap())?,
            kind => {
                // TODO: def and type formatting
                self.out.write_all(node.to_string().as_bytes())?;
                tracing::warn!(?kind, "unformatted syntax node");
            }
        }

        Ok(())
    }
}
