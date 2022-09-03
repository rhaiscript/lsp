use std::io::{self, Write};

use rhai_rowan::{
    ast::{
        AstNode, ExportTarget, Expr, ExprBlock, ExprConst, ExprContinue, ExprIf, ExprLet,
        LitStrTemplateSegment,
    },
    syntax::{
        SyntaxElement,
        SyntaxKind::{self, *},
    },
    T,
};

use crate::{
    algorithm::Formatter,
    source::needs_stmt_separator,
    util::{break_count, ScopedStatic},
};

impl<S: Write> Formatter<S> {
    pub(crate) fn fmt_expr(&mut self, expr: Expr) -> io::Result<()> {
        let syntax = expr.syntax();

        match expr {
            Expr::Ident(expr) => {
                self.fmt_expr_ident(expr)?;
            }
            Expr::Path(expr) => {
                self.fmt_expr_path(expr)?;
            }
            Expr::Lit(expr) => {
                self.fmt_expr_lit(expr)?;
            }
            Expr::Let(expr) => {
                self.fmt_expr_let(expr)?;
            }
            Expr::Const(expr) => {
                self.fmt_expr_const(expr)?;
            }
            Expr::Block(expr) => {
                self.fmt_expr_block(expr, false, false)?;
            }
            Expr::Unary(expr) => {
                self.fmt_expr_unary(expr)?;
            }
            Expr::Binary(expr) => {
                self.fmt_expr_binary(expr)?;
            }
            Expr::Paren(expr) => {
                self.fmt_expr_paren(expr)?;
            }
            Expr::Array(expr) => {
                self.fmt_expr_array(expr)?;
            }
            Expr::Index(expr) => {
                self.fmt_expr_index(expr)?;
            }
            Expr::Object(expr) => {
                self.fmt_expr_object(expr)?;
            }
            Expr::Call(expr) => {
                self.fmt_expr_call(expr)?;
            }
            Expr::Closure(expr) => {
                self.fmt_expr_closure(expr)?;
            }
            Expr::If(expr) => {
                self.fmt_expr_if(expr, false)?;
            }
            Expr::Loop(expr) => {
                self.fmt_expr_loop(expr)?;
            }
            Expr::For(expr) => {
                self.fmt_expr_for(expr)?;
            }
            Expr::While(expr) => {
                self.fmt_expr_while(expr)?;
            }
            Expr::Break(expr) => {
                self.fmt_expr_break(expr)?;
            }
            Expr::Continue(expr) => {
                self.fmt_expr_continue(expr)?;
            }
            Expr::Switch(expr) => {
                self.fmt_expr_switch(expr)?;
            }
            Expr::Return(expr) => {
                self.fmt_expr_return(expr)?;
            }
            Expr::Fn(expr) => {
                self.fmt_expr_fn(expr)?;
            }
            Expr::Export(expr) => {
                self.fmt_expr_export(expr)?;
            }
            Expr::Import(expr) => {
                self.fmt_expr_import(expr)?;
            }
            Expr::Try(expr) => {
                self.fmt_expr_try(expr)?;
            }
            Expr::Throw(expr) => {
                self.fmt_expr_throw(expr)?;
            }
        }

        if let Some(child) = syntax.first_child() {
            self.trailing_comments_after(&child, true)?;
        }

        Ok(())
    }

    pub(crate) fn fmt_expr_throw(
        &mut self,
        expr: rhai_rowan::ast::ExprThrow,
    ) -> Result<(), io::Error> {
        self.word("throw")?;
        if let Some(expr) = expr.expr() {
            self.nbsp()?;
            self.fmt_expr(expr)?;
        };
        Ok(())
    }

    pub(crate) fn fmt_expr_try(&mut self, expr: rhai_rowan::ast::ExprTry) -> Result<(), io::Error> {
        self.word("try ")?;
        if let Some(body) = expr.try_block() {
            self.fmt_expr_block(body, true, false)?;
        }
        self.word(" catch ")?;
        if let Some(param_list) = expr.catch_params() {
            self.word("(")?;
            self.cbox(1);
            self.zerobreak();

            let count = param_list.params().count();

            for (i, param) in param_list.params().enumerate() {
                if let Some(ident) = param.ident_token() {
                    self.word(ident.static_text())?;
                }
                self.trailing_comma(i + 1 == count)?;
            }

            self.word(")")?;
            self.space();
            self.offset(-1);
            self.end();
            self.neverbreak();
        }
        if let Some(body) = expr.catch_block() {
            self.fmt_expr_block(body, true, false)?;
        };
        Ok(())
    }

    pub(crate) fn fmt_expr_import(
        &mut self,
        expr: rhai_rowan::ast::ExprImport,
    ) -> Result<(), io::Error> {
        self.word("import ")?;
        if let Some(expr) = expr.expr() {
            self.fmt_expr(expr)?;
        }
        if let Some(alias) = expr.alias() {
            self.word(" as ")?;
            self.word(alias.static_text())?;
        };
        Ok(())
    }

    pub(crate) fn fmt_expr_export(
        &mut self,
        expr: rhai_rowan::ast::ExprExport,
    ) -> Result<(), io::Error> {
        self.word("export ")?;
        if let Some(target) = expr.export_target() {
            match target {
                ExportTarget::ExprLet(expr) => self.fmt_expr_let(expr)?,
                ExportTarget::ExprConst(expr) => self.fmt_expr_const(expr)?,
                ExportTarget::Ident(ident) => {
                    if let Some(ident) = ident.ident_token() {
                        self.word(ident.static_text())?;
                    }

                    if let Some(alias) = ident.alias() {
                        self.word(" as ")?;
                        self.word(alias.static_text())?;
                    }
                }
            }
        };

        Ok(())
    }

    pub(crate) fn fmt_expr_fn(&mut self, expr: rhai_rowan::ast::ExprFn) -> Result<(), io::Error> {
        if expr.kw_private_token().is_some() {
            self.word("private ")?;
        }
        self.word("fn ")?;
        if let Some(ident) = expr.ident_token() {
            self.word(ident.static_text())?;
        }
        self.word("(")?;
        self.cbox(1);
        self.zerobreak();
        if let Some(param_list) = expr.param_list() {
            let count = param_list.params().count();

            for (i, param) in param_list.params().enumerate() {
                if let Some(ident) = param.ident_token() {
                    self.word(ident.static_text())?;
                }
                self.trailing_comma(i + 1 == count)?;
            }
        }
        self.word(")")?;
        self.space();
        self.offset(-1);
        self.end();
        self.neverbreak();
        if let Some(body) = expr.body() {
            self.fmt_expr_block(body, true, false)?;
        };
        Ok(())
    }

    pub(crate) fn fmt_expr_return(
        &mut self,
        expr: rhai_rowan::ast::ExprReturn,
    ) -> Result<(), io::Error> {
        self.word("return")?;
        if let Some(expr) = expr.expr() {
            self.nbsp()?;
            self.fmt_expr(expr)?;
        };
        Ok(())
    }

    pub(crate) fn fmt_expr_switch(
        &mut self,
        expr: rhai_rowan::ast::ExprSwitch,
    ) -> Result<(), io::Error> {
        self.word("switch ")?;
        if let Some(expr) = expr.expr() {
            self.fmt_expr(expr)?;
        }
        self.nbsp()?;
        self.word("{")?;
        self.cbox(1);
        self.hardbreak();
        if let Some(arm_list) = expr.switch_arm_list() {
            let arm_count = arm_list.arms().count();
            for (i, arm) in arm_list.arms().enumerate() {
                if i != 0 {
                    self.hardbreak();
                }

                if let Some(pat) = arm.pattern_expr() {
                    self.fmt_expr(pat)?;
                }

                if let Some(cond) = arm.condition().and_then(|c| c.expr()) {
                    self.word(" if ")?;
                    self.fmt_expr(cond)?;
                }

                self.word(" => ")?;

                if let Some(expr) = arm.value_expr() {
                    self.fmt_expr(expr)?;
                }

                let is_last = i + 1 == arm_count;
                self.trailing_comma(is_last)?;
            }
        }
        self.offset(-1);
        self.end();
        self.word("}")?;
        Ok(())
    }

    pub(crate) fn fmt_expr_continue(&mut self, _expr: ExprContinue) -> Result<(), io::Error> {
        self.word("continue")?;
        Ok(())
    }

    pub(crate) fn fmt_expr_break(
        &mut self,
        expr: rhai_rowan::ast::ExprBreak,
    ) -> Result<(), io::Error> {
        self.word("break")?;
        if let Some(expr) = expr.expr() {
            self.nbsp()?;
            self.fmt_expr(expr)?;
        };
        Ok(())
    }

    pub(crate) fn fmt_expr_while(
        &mut self,
        expr: rhai_rowan::ast::ExprWhile,
    ) -> Result<(), io::Error> {
        self.cbox(0);
        self.word("while ")?;
        if let Some(cond) = expr.expr() {
            self.fmt_expr(cond)?;
        }
        self.nbsp()?;
        if let Some(body) = expr.loop_body() {
            self.fmt_expr_block(body, true, false)?;
        }
        self.end();
        Ok(())
    }

    pub(crate) fn fmt_expr_loop(
        &mut self,
        expr: rhai_rowan::ast::ExprLoop,
    ) -> Result<(), io::Error> {
        self.word("loop ")?;
        if let Some(expr) = expr.loop_body() {
            self.fmt_expr_block(expr, false, false)?;
        };
        Ok(())
    }

    pub(crate) fn fmt_expr_for(&mut self, expr: rhai_rowan::ast::ExprFor) -> Result<(), io::Error> {
        self.word("for ")?;
        if let Some(pat) = expr.pat() {
            let ident_count = pat.idents().count();

            if ident_count == 1 {
                self.word(pat.idents().next().unwrap().static_text())?;
            } else {
                self.word("(")?;
                self.cbox(1);
                self.zerobreak();

                for (i, ident) in pat.idents().enumerate() {
                    self.word(ident.static_text())?;
                    self.trailing_comma(i + 1 == ident_count)?;
                }
                self.offset(-1);
                self.end();
                self.word(")")?;
            }

            self.nbsp()?;
        }
        self.word("in ")?;
        self.cbox(0);
        if let Some(expr) = expr.iterable() {
            self.fmt_expr(expr)?;
        }
        self.nbsp()?;
        self.neverbreak();
        if let Some(block) = expr.loop_body() {
            self.fmt_expr_block(block, true, false)?;
        }
        self.end();
        Ok(())
    }

    pub(crate) fn fmt_expr_closure(
        &mut self,
        expr: rhai_rowan::ast::ExprClosure,
    ) -> Result<(), io::Error> {
        self.cbox(1);
        self.word("|")?;
        self.zerobreak();
        if let Some(param_list) = expr.param_list() {
            let count = param_list.params().count();

            for (i, param) in param_list.params().enumerate() {
                if let Some(ident) = param.ident_token() {
                    self.word(ident.static_text())?;
                }
                self.trailing_comma(i + 1 == count)?;
            }
        }
        self.word("|")?;
        self.space();
        self.offset(-1);
        self.end();
        self.neverbreak();
        if let Some(body) = expr.body() {
            self.fmt_expr(body)?;
        };
        Ok(())
    }

    pub(crate) fn fmt_expr_call(
        &mut self,
        expr: rhai_rowan::ast::ExprCall,
    ) -> Result<(), io::Error> {
        if let Some(base) = expr.expr() {
            self.fmt_expr(base)?;
        }
        self.word("(")?;
        self.cbox(1);
        self.zerobreak();
        if let Some(args) = expr.arg_list() {
            let count = args.arguments().count();

            for (i, arg) in args.arguments().enumerate() {
                self.fmt_expr(arg)?;
                self.trailing_comma(i + 1 == count)?;
            }
        }
        self.offset(-1);
        self.end();
        self.word(")")?;
        Ok(())
    }

    pub(crate) fn fmt_expr_object(
        &mut self,
        expr: rhai_rowan::ast::ExprObject,
    ) -> Result<(), io::Error> {
        let count = expr.fields().count();

        if count == 0 {
            return self.word("#{}");
        }

        let always_break = expr
            .syntax()
            .descendants_with_tokens()
            .any(|c| match c.kind() {
                COMMENT_LINE | COMMENT_LINE_DOC => true,
                WHITESPACE => break_count(c.as_token().unwrap()) > 0,
                _ => false,
            });

        self.word("#{")?;
        self.cbox(1);

        if always_break {
            self.hardbreak();
        } else {
            self.space();
        }

        for (i, field) in expr.fields().enumerate() {
            self.ibox(0);
            if let Some(prop) = field.property() {
                self.word(prop.static_text())?;
            }

            self.word(":")?;
            self.space();
            self.offset(1);
            if let Some(expr) = field.expr() {
                self.fmt_expr(expr)?;
            }
            self.end();

            let last = i + 1 == count;

            self.trailing_comma_or_space(last)?;
        }
        self.offset(-1);
        self.end();
        self.word("}")?;
        Ok(())
    }

    pub(crate) fn fmt_expr_index(
        &mut self,
        expr: rhai_rowan::ast::ExprIndex,
    ) -> Result<(), io::Error> {
        if let Some(base) = expr.base() {
            self.fmt_expr(base)?;
        }
        self.word("[")?;
        if let Some(idx) = expr.index() {
            self.fmt_expr(idx)?;
        }
        self.word("]")?;
        Ok(())
    }

    pub(crate) fn fmt_expr_array(
        &mut self,
        expr: rhai_rowan::ast::ExprArray,
    ) -> Result<(), io::Error> {
        self.word("[")?;
        self.cbox(1);
        self.zerobreak();
        let count = expr.values().count();
        for (i, value) in expr.values().enumerate() {
            self.fmt_expr(value)?;
            self.trailing_comma(i + 1 == count)?;
        }
        self.offset(-1);
        self.end();
        self.word("]")?;
        Ok(())
    }

    pub(crate) fn fmt_expr_paren(
        &mut self,
        expr: rhai_rowan::ast::ExprParen,
    ) -> Result<(), io::Error> {
        self.word("(")?;
        if let Some(expr) = expr.expr() {
            self.fmt_expr(expr)?;
        }
        self.word(")")?;
        Ok(())
    }

    pub(crate) fn fmt_expr_binary(
        &mut self,
        expr: rhai_rowan::ast::ExprBinary,
    ) -> Result<(), io::Error> {
        self.ibox(1);
        self.ibox(-1);
        if let Some(lhs) = expr.lhs() {
            self.fmt_expr(lhs)?;
        }
        self.end();

        if let Some(op) = expr.op_token().map(|t| t.kind()) {
            self.break_before_op(op)?;
        }
        if let Some(op) = expr.op_token() {
            self.word(op.static_text())?;
        }
        if let Some(op) = expr.op_token().map(|t| t.kind()) {
            self.break_after_op(op)?;
        }
        if let Some(rhs) = expr.rhs() {
            self.fmt_expr(rhs)?;
        }
        self.end();
        Ok(())
    }

    pub(crate) fn fmt_expr_unary(
        &mut self,
        expr: rhai_rowan::ast::ExprUnary,
    ) -> Result<(), io::Error> {
        if let Some(op) = expr.op_token() {
            self.word(op.static_text())?;
            if op.kind() == IDENT {
                self.nbsp()?;
            }
        }
        if let Some(expr) = expr.expr() {
            self.fmt_expr(expr)?;
        };
        Ok(())
    }

    pub(crate) fn fmt_expr_lit(&mut self, expr: rhai_rowan::ast::ExprLit) -> Result<(), io::Error> {
        if let Some(lit) = expr.lit() {
            if let Some(t) = lit.lit_token() {
                self.word(t.static_text())?;
            } else if let Some(template) = lit.lit_str_template() {
                for segment in template.segments() {
                    match segment {
                        LitStrTemplateSegment::LitStr(s) => {
                            self.word(s.static_text())?;
                        }
                        LitStrTemplateSegment::Interpolation(interpolation) => {
                            let count = interpolation.statements().count();

                            match count {
                                0 => {
                                    self.word("${}")?;
                                }
                                _ => {
                                    self.word("${")?;
                                    self.ibox(0);
                                    self.zerobreak();

                                    for (idx, statement) in interpolation.statements().enumerate() {
                                        if idx != 0 {
                                            self.word(";")?;
                                            self.space();
                                        }

                                        self.fmt_stmt(statement)?;
                                    }

                                    self.zerobreak();
                                    self.end();
                                    self.word("}")?;
                                }
                            }
                        }
                    }
                }
            }
        };
        Ok(())
    }

    pub(crate) fn fmt_expr_path(
        &mut self,
        expr: rhai_rowan::ast::ExprPath,
    ) -> Result<(), io::Error> {
        if let Some(path) = expr.path() {
            self.fmt_path(path)?;
        };
        Ok(())
    }

    pub(crate) fn fmt_expr_ident(
        &mut self,
        expr: rhai_rowan::ast::ExprIdent,
    ) -> Result<(), io::Error> {
        if let Some(t) = expr.ident_token() {
            self.word(t.static_text())?;
        };
        Ok(())
    }

    pub(crate) fn fmt_expr_const(&mut self, expr: ExprConst) -> Result<(), io::Error> {
        self.ibox(1);
        self.word("const")?;
        self.ibox(-1);
        self.nbsp()?;
        if let Some(ident) = expr.ident_token() {
            self.word(ident.static_text())?;
        }
        self.end();
        if let Some(rhs) = expr.expr() {
            self.word(" = ")?;
            self.fmt_expr(rhs)?;
        }
        self.end();
        Ok(())
    }

    pub(crate) fn fmt_expr_let(&mut self, expr: ExprLet) -> Result<(), io::Error> {
        self.ibox(1);
        self.word("let")?;
        self.ibox(-1);
        self.nbsp()?;
        if let Some(ident) = expr.ident_token() {
            self.word(ident.static_text())?;
        }
        self.end();
        if let Some(rhs) = expr.expr() {
            self.word(" = ")?;
            self.fmt_expr(rhs)?;
        }
        self.end();
        Ok(())
    }

    pub(crate) fn fmt_expr_if(&mut self, expr: ExprIf, no_cbox: bool) -> Result<(), io::Error> {
        self.word("if ")?;
        if !no_cbox {
            self.cbox(1);
        }

        if let Some(cond) = expr.expr() {
            self.fmt_expr(cond)?;
        }
        self.nbsp()?;

        if let Some(then) = expr.then_branch() {
            self.fmt_expr_block(then, true, true)?;
        }

        if let Some(else_if_branch) = expr.else_if_branch() {
            self.word(" else ")?;
            self.fmt_expr_if(else_if_branch, true)?;
        }
        if let Some(else_branch) = expr.else_branch() {
            self.word(" else ")?;
            self.fmt_expr_block(else_branch, true, true)?;
        }

        if !no_cbox {
            self.end();
        }

        Ok(())
    }

    pub(crate) fn fmt_expr_block(
        &mut self,
        expr: ExprBlock,
        mut always_break: bool,
        no_cbox: bool,
    ) -> io::Result<()> {
        always_break = always_break
            || expr
                .syntax()
                .descendants_with_tokens()
                .any(|c| matches!(c.kind(), COMMENT_LINE | COMMENT_LINE_DOC));

        let syntax = expr.syntax();

        if !no_cbox {
            self.cbox(1);
        }

        self.word("{")?;
        match expr.statements().count() {
            0 => {
                // Special case where the block
                // contains comments but nothing else.
                let comments = expr
                    .syntax()
                    .children_with_tokens()
                    .filter_map(SyntaxElement::into_token)
                    .filter(|t| matches!(t.kind(), COMMENT_BLOCK | COMMENT_LINE))
                    .collect::<Vec<_>>();

                if !comments.is_empty() {
                    self.space();

                    let mut first = true;
                    for comment in comments {
                        if !first {
                            self.hardbreak();
                        }
                        first = false;
                        self.word(comment.static_text().trim())?;
                    }
                    self.hardbreak();
                    self.offset(-1);
                }
            }
            1 => {
                self.space();
                self.leading_comments_in(&syntax)?;

                let stmt = expr.statements().next().unwrap();

                let had_sep = stmt
                    .syntax()
                    .children_with_tokens()
                    .any(|c| c.kind() == T![";"]);

                if let Some(item) = stmt.item() {
                    let needs_sep = needs_stmt_separator(&item);
                    self.fmt_item(item)?;

                    if had_sep && needs_sep {
                        self.word(";")?;
                    }

                    self.trailing_comments_after(&stmt.syntax(), true)?;
                }

                if always_break {
                    self.hardbreak();
                    self.offset(-1);
                } else {
                    self.space();
                }
            }
            _ => {
                self.space();
                self.leading_comments_in(&syntax)?;

                let count = expr.statements().count();

                let mut first = true;
                let mut needs_sep = false;
                for (idx, stmt) in expr.statements().enumerate() {
                    let item = match stmt.item() {
                        Some(item) => item,
                        _ => continue,
                    };

                    let syntax = stmt.syntax();

                    if !first {
                        if needs_sep {
                            self.word(";")?;
                        }

                        self.comments_before_or_hardbreak(&syntax)?;
                    }
                    first = false;

                    needs_sep = needs_stmt_separator(&item);
                    self.fmt_item(item)?;

                    let last = count == idx + 1;

                    if last {
                        let had_sep = stmt
                            .syntax()
                            .children_with_tokens()
                            .any(|c| c.kind() == T![";"]);

                        if had_sep && needs_sep {
                            self.word(";")?;
                        }

                        self.trailing_comments_after(&syntax, false)?;
                        self.hardbreak();
                    }
                }

                self.offset(-1);
            }
        }
        self.word("}")?;
        if !no_cbox {
            self.end();
        }
        Ok(())
    }

    fn break_before_op(&mut self, kind: SyntaxKind) -> io::Result<()> {
        match kind {
            T![".."] | T!["..="] => {}
            T!["."] | T!["?."] => self.zerobreak(),
            _ => self.space(),
        }

        Ok(())
    }

    fn break_after_op(&mut self, kind: SyntaxKind) -> io::Result<()> {
        match kind {
            T![".."] | T!["..="] | T!["."] | T!["?."] => {}
            _ => self.nbsp()?,
        }

        Ok(())
    }
}
