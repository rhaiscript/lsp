//! Textual representation of the HIR for debugging purposes only.
//!
//! # Format
//!
//! The output format is designed to be somewhat
//! readable, but is not intended to be too pretty.
//! It is not stable and should not be relied on in any way.
//!
//! ## Slots
//!
//! The HIR uses [`SlotMap`](slotmap::SlotMap) to store most of the information,
//! the slots are represented via the `@<index>:<version>` format
//! where `<index>` is the current index of the item inside the map,
//! and `<version>` is the generation in case the slot was reused.
//!
//! Modules, scopes, symbols, types, and sources use separate maps,
//! so there can be overlaps between these indexes.
//!
//! Having two equal indexes of the same type (e.g. symbol)
//! with different versions indicates a bug where the HIR expects
//! a previous version of a symbol that has been overwritten.
//!
//! ## Symbols
//!
//! All symbols start with `$` followed by an optional symbol type
//! name (e.g. `Fn`) and slot index.
//!
//! ## Scopes
//!
//! Similarly to the Rhai syntax, scopes are represented via brackets `{}`
//! optionally preceded by their slot index.
//!
//! ## References
//!
//! Reference targets are marked as `<alias> => <target>` where `<target>`
//! is the target type and slot index and `<alias>` is an optional alias.
//!
//! ## Errors
//!
//! Missing values are typically shown in the form of `MISSING <value>`
//! that suggests that the syntax tree was incomplete.
//!
//! `!MISSING <item>` notations suggest a bug in the HIR.
//!
//! ## Parents
//!
//! To uncover bugs, in some cases it is useful to print parents
//! of items (e.g. scopes of symbols), parents use the `^<target>` notation.
//!
//! ## Duplicate symbols
//!
//! You might notice that some symbols might appear more than once,
//! this is *not* a bug. By design some symbols do not have a scope,
//! for example symbols in arrays (`["foo", "bar"]`) share
//! the scope the array itself was defined in.
//! When the HIR is printed, we still want to show that the symbols
//! in addition to being in the scope are also part of the array,
//! thus the representation will end up something like this:
//!
//! ```text
//! $sym1
//! $sym2
//! $array [
//!   $sym1
//!   $sym2
//! ]
//! ```
//!
//! # Examples
//!
//! The following script:
//!
//! ```rhai
//! fn foo(p1, p2) {
//!   let a = 2;
//!   let b = a;
//!   return b + p2;
//! }
//! ```
//!
//! Will produce the HIR representation below:
//!
//! ```text
//! source@1:1 Def rhai-virtual:/// => module@1:1
//! source@2:1 Script test:///root.rhai => module@2:1
//! protected module@1:1 static @1:1{}
//! module@2:1 test:///root.rhai @2:1{
//!   export $Fn@11:1 foo @3:1{
//!     $Decl@1:1 param p1
//!     $Decl@2:1 param p2
//!     $Decl@4:1 let a = @4:1{
//!       $Lit@3:1 2
//!     }
//!     $Decl@6:1 let b = @5:1{
//!       $Ref@5:1 a => $@4:1
//!     }
//!     $Binary@9:1 +
//!       $Ref@7:1 b => $@6:1
//!       $Ref@8:1 p2 => $@2:1
//!     $Return@10:1
//!       $Binary@9:1 +
//!         $Ref@7:1 b => $@6:1
//!         $Ref@8:1 p2 => $@2:1
//!   }
//! }
//! ```

use slotmap::{Key, KeyData};

use crate::{
    scope::ScopeParent,
    source::Source,
    symbol::{BinaryOpKind, ReferenceTarget, SymbolKind, VirtualSymbol},
    Hir, Module, Scope, Symbol,
};
use std::fmt::{self, Write};

macro_rules! windent {
    ($hir_fmt:expr, $w:expr) => {
        {
            $hir_fmt.write_indent($w)
        }
    };

    ($hir_fmt:expr, $w:expr, $($args:tt)*) => {
        {
            $hir_fmt.write_indent($w)?;
            write!($w, $($args)*)
        }
    };
}

macro_rules! windentln {
    ($hir_fmt:expr, $w:expr, $($args:tt)*) => {
        {
            $hir_fmt.write_indent($w)?;
            writeln!($w, $($args)*)
        }
    };
}

#[derive(Clone, Copy)]
#[must_use]
#[allow(clippy::struct_excessive_bools)]
pub struct HirFmt<'h> {
    hir: &'h Hir,
    indent_level: usize,
    include_slots: bool,
    include_sources: bool,
    include_parents: bool,
    print_all: bool,
}

impl fmt::Debug for Hir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut hir_fmt = HirFmt::new(self).with_slots();

        if f.alternate() {
            hir_fmt.include_sources = true;
            hir_fmt.include_parents = true;
            hir_fmt.print_all = true;
        }

        <_ as fmt::Display>::fmt(&hir_fmt, f)
    }
}

impl fmt::Display for HirFmt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for s in self.hir.sources.keys() {
            self.fmt_source(f, s)?;
            writeln!(f)?;
        }

        for m in self.hir.modules.keys() {
            self.fmt_module(f, m)?;
            writeln!(f)?;
        }

        if self.print_all {
            for s in self.hir.symbols.keys() {
                self.fmt_symbol(f, s)?;
                writeln!(f)?;
            }

            for s in self.hir.scopes.keys() {
                self.fmt_scope(f, s)?;
                writeln!(f)?;
            }
        }

        Ok(())
    }
}

impl<'h> HirFmt<'h> {
    pub fn new(hir: &'h Hir) -> Self {
        Self {
            hir,
            indent_level: 0,
            include_slots: true,
            include_sources: false,
            include_parents: false,
            print_all: false,
        }
    }

    pub fn with_slots(mut self) -> Self {
        self.include_slots = true;
        self
    }

    pub fn with_source(mut self) -> Self {
        self.include_sources = true;
        self
    }

    pub fn with_parents(mut self) -> Self {
        self.include_parents = true;
        self
    }

    pub fn with_all(self) -> Self {
        self.with_slots().with_parents().with_source()
    }

    pub fn module(self, module: Module) -> ModuleFmt<'h> {
        ModuleFmt { module, f: self }
    }

    pub fn symbol(self, symbol: Symbol) -> SymbolFmt<'h> {
        SymbolFmt { symbol, f: self }
    }

    pub fn scope(self, scope: Scope) -> ScopeFmt<'h> {
        ScopeFmt { scope, f: self }
    }

    pub fn source(self, source: Source) -> SourceFmt<'h> {
        SourceFmt { source, f: self }
    }

    fn incr_indent(self) -> Self {
        Self {
            indent_level: self.indent_level + 1,
            ..self
        }
    }

    fn write_indent(&self, writer: &mut impl Write) -> fmt::Result {
        for _ in 0..self.indent_level {
            writer.write_str("  ")?;
        }

        Ok(())
    }

    fn fmt_source(&self, f: &mut fmt::Formatter, source: Source) -> fmt::Result {
        match self.hir.sources.get(source) {
            Some(s) => {
                write!(
                    f,
                    "source{slot} {kind:?} {url} => module{m}",
                    slot = if self.include_slots {
                        KeyDataFmt(source.data()).to_string()
                    } else {
                        String::new()
                    },
                    kind = s.kind,
                    url = s.url,
                    m = KeyDataFmt(s.module.data())
                )?;
            }
            None => {
                write!(f, "!MISSING SOURCE{}", KeyDataFmt(source.data()))?;
            }
        }

        Ok(())
    }

    fn fmt_module(&self, f: &mut fmt::Formatter, module: Module) -> fmt::Result {
        match self.hir.modules.get(module) {
            Some(m) => {
                write!(
                    f,
                    "{protected}module{slot} {kind} ",
                    slot = if self.include_slots {
                        KeyDataFmt(module.data()).to_string()
                    } else {
                        String::new()
                    },
                    protected = if m.protected { "protected " } else { "" },
                    kind = m.kind
                )?;

                self.fmt_scope(f, m.scope)?;
            }
            None => {
                write!(f, "!MISSING MODULE{}", KeyDataFmt(module.data()))?;
            }
        }

        Ok(())
    }

    fn fmt_scope(&self, f: &mut fmt::Formatter, scope: Scope) -> fmt::Result {
        match self.hir.scopes.get(scope) {
            Some(s) => {
                let mut extra = if self.include_slots {
                    KeyDataFmt(scope.data()).to_string()
                } else {
                    String::new()
                };

                if self.include_parents {
                    match s.parent {
                        Some(parent) => match parent {
                            ScopeParent::Scope(s) => {
                                write!(&mut extra, "(^scope{})", KeyDataFmt(s.data())).unwrap();
                            }
                            ScopeParent::Symbol(s) => {
                                write!(&mut extra, "(^${})", KeyDataFmt(s.data())).unwrap();
                            }
                        },
                        None => {
                            extra += "(^-)";
                        }
                    }
                }

                if s.is_empty() {
                    return write!(f, "{extra}{{}}");
                }

                writeln!(f, "{extra}{{")?;

                let fmt_child = self.incr_indent();

                for symbol in s.iter_symbols() {
                    fmt_child.fmt_symbol(f, symbol)?;
                    writeln!(f)?;
                }

                windent!(self, f, "}}")?;
            }
            None => {
                write!(f, "{{ !MISSING SCOPE{} }}", KeyDataFmt(scope.data()))?;
            }
        }

        Ok(())
    }

    fn fmt_symbol(&self, f: &mut fmt::Formatter, symbol: Symbol) -> fmt::Result {
        let data = match self.hir.symbols.get(symbol) {
            Some(sym) => sym,
            None => {
                windentln!(self, f, "$!MISSING{} ", KeyDataFmt(symbol.data()))?;
                return Ok(());
            }
        };

        let slot = if self.include_slots {
            KeyDataFmt(symbol.data()).to_string()
        } else {
            String::new()
        };

        windent!(
            self,
            f,
            "{export}${symbol_name}{slot}",
            export = if data.export { "export " } else { "" },
            symbol_name = <&str>::from(&data.kind)
        )?;

        if self.include_parents {
            write!(f, " (^scope{})", KeyDataFmt(data.parent_scope.data()))?;
        }

        if self.include_sources {
            write!(f, " (")?;
            if let Some(source) = data.source.source {
                write!(f, "source{} ", KeyDataFmt(source.data()))?;
            }

            if let Some(range) = data.source.text_range {
                write!(f, "{range:?}")?;
            }

            write!(f, ")")?;
        }

        match &data.kind {
            SymbolKind::Block(block) => {
                write!(f, " ")?;
                self.fmt_scope(f, block.scope)?;
            }
            SymbolKind::Fn(fn_data) => {
                write!(
                    f,
                    " {def}{get}{set}{name} ",
                    def = if fn_data.is_def { "def " } else { "" },
                    set = if fn_data.setter { "set " } else { "" },
                    get = if fn_data.getter { "get " } else { "" },
                    name = fn_data.name
                )?;

                self.fmt_scope(f, fn_data.scope)?;
            }
            SymbolKind::Decl(decl) => {
                write!(
                    f,
                    " {cst}{name}",
                    cst = if decl.is_param {
                        "param "
                    } else if decl.is_const {
                        "const "
                    } else {
                        "let "
                    },
                    name = decl.name
                )?;

                if let Some(value_scope) = decl.value_scope {
                    write!(f, " = ")?;
                    self.fmt_scope(f, value_scope)?;
                }
            }
            SymbolKind::Ref(r) => {
                write!(f, " {}", r.name)?;

                if let Some(target) = r.target {
                    match target {
                        ReferenceTarget::Symbol(sym) => {
                            write!(f, " => ${}", KeyDataFmt(sym.data()))?;
                        }
                        ReferenceTarget::Module(m) => {
                            write!(f, " => module{}", KeyDataFmt(m.data()))?;
                        }
                    }
                }
            }
            SymbolKind::Path(p) => {
                let indented = self.incr_indent();
                writeln!(f)?;

                let mut first = true;
                for &segment in &p.segments {
                    if !first {
                        writeln!(f)?;
                    }
                    first = false;

                    indented.fmt_symbol(f, segment)?;
                }
            }
            SymbolKind::Lit(lit) => {
                write!(f, " {}", lit.value)?;

                if !lit.interpolated_scopes.is_empty() {
                    writeln!(f)?;
                }

                let mut first = true;
                for &scope in &lit.interpolated_scopes {
                    if !first {
                        writeln!(f)?;
                    }
                    first = false;

                    windent!(self, f, "template ")?;
                    self.fmt_scope(f, scope)?;
                }
            }
            SymbolKind::Unary(op) => {
                let indented = self.incr_indent();
                writeln!(
                    f,
                    " {op}",
                    op = if op.lookup_text.is_empty() {
                        match op.op {
                            Some(syntax) => <&str>::from(syntax),
                            None => "<unknown syntax>",
                        }
                    } else {
                        &op.lookup_text
                    }
                )?;

                if let Some(rhs) = op.rhs {
                    indented.fmt_symbol(f, rhs)?;
                } else {
                    windent!(indented, f, "MISSING RHS")?;
                }
            }
            SymbolKind::Binary(op) => {
                let indented = self.incr_indent();

                writeln!(
                    f,
                    " {op}",
                    op = if op.lookup_text.is_empty() {
                        match &op.op {
                            Some(bin) => match &bin {
                                BinaryOpKind::Regular(syntax) => <&str>::from(syntax),
                                BinaryOpKind::Custom(c) => c.name.as_ref(),
                            },
                            None => "UNKNOWN SYNTAX",
                        }
                    } else {
                        &op.lookup_text
                    }
                )?;

                if let Some(lhs) = op.lhs {
                    indented.fmt_symbol(f, lhs)?;
                } else {
                    windent!(indented, f, "MISSING LHS")?;
                }

                writeln!(f)?;

                if let Some(rhs) = op.rhs {
                    indented.fmt_symbol(f, rhs)?;
                } else {
                    windent!(indented, f, "MISSING RHS")?;
                }
            }
            SymbolKind::Array(arr) => {
                if arr.values.is_empty() {
                    write!(f, "[]")?;
                } else {
                    writeln!(f, " [")?;

                    let indented = self.incr_indent();

                    for &val in &arr.values {
                        indented.fmt_symbol(f, val)?;
                        writeln!(f)?;
                    }
                    windent!(self, f, "]")?;
                }
            }
            SymbolKind::Index(idx) => {
                let indented = self.incr_indent();
                writeln!(f)?;

                if let Some(base) = idx.base {
                    indented.fmt_symbol(f, base)?;
                } else {
                    windent!(indented, f, "MISSING IDX BASE")?;
                }

                writeln!(f)?;

                if let Some(idx) = idx.index {
                    indented.fmt_symbol(f, idx)?;
                } else {
                    windent!(indented, f, "MISSING IDX")?;
                }
            }
            SymbolKind::Object(obj) => {
                if obj.fields.is_empty() {
                    write!(f, "#{{}}")?;
                } else {
                    writeln!(f, " #{{")?;

                    let indented = self.incr_indent();

                    for (key, field) in &obj.fields {
                        windentln!(indented, f, "{key}:")?;
                        if let Some(val) = field.value {
                            indented.fmt_symbol(f, val)?;
                        } else {
                            windent!(indented, f, "MISSING VALUE")?;
                        }
                        writeln!(f)?;
                    }
                    windent!(self, f, "}}")?;
                }
            }
            SymbolKind::Call(call) => {
                let indented = self.incr_indent();
                writeln!(f)?;

                if let Some(lhs) = call.lhs {
                    indented.fmt_symbol(f, lhs)?;
                } else {
                    windent!(indented, f, "MISSING CALL LHS")?;
                }

                if !call.arguments.is_empty() {
                    writeln!(f)?;
                }

                let args = indented.incr_indent();

                let mut first = true;
                for &arg in &call.arguments {
                    if !first {
                        writeln!(f)?;
                    }
                    first = false;
                    args.fmt_symbol(f, arg)?;
                }
            }
            SymbolKind::Closure(closure) => {
                write!(f, " ")?;
                self.fmt_scope(f, closure.scope)?;
            }
            SymbolKind::If(if_sym) => {
                if !if_sym.branches.is_empty() {
                    writeln!(f)?;
                }

                let indented = self.incr_indent();

                let mut first = true;
                for (condition, branch) in &if_sym.branches {
                    if !first {
                        writeln!(f)?;
                    }
                    first = false;

                    windentln!(indented, f, "if")?;
                    match condition {
                        Some(c) => {
                            indented.fmt_symbol(f, *c)?;
                            writeln!(f)?;
                        }
                        None => windentln!(indented, f, "MISSING CONDITION")?,
                    }
                    windentln!(indented, f, "then")?;
                    indented.fmt_scope(f, *branch)?;
                }
            }
            SymbolKind::Loop(l) => {
                write!(f, " ")?;
                self.fmt_scope(f, l.scope)?;
            }
            SymbolKind::For(fr) => {
                write!(f, " ")?;
                self.fmt_scope(f, fr.scope)?;
            }
            SymbolKind::While(whl) => {
                writeln!(f)?;

                let indented = self.incr_indent();

                windentln!(indented, f, "while")?;
                if let Some(cond) = whl.condition {
                    indented.fmt_symbol(f, cond)?;
                    writeln!(f)?;
                } else {
                    windentln!(indented, f, "MISSING CONDITION")?;
                }
                windentln!(indented, f, "do")?;
                indented.fmt_scope(f, whl.scope)?;
            }
            SymbolKind::Break(br) => {
                if let Some(br_val) = br.expr {
                    let indented = self.incr_indent();
                    writeln!(f)?;
                    indented.fmt_symbol(f, br_val)?;
                }
            }
            SymbolKind::Return(ret) => {
                if let Some(ret_val) = ret.expr {
                    let indented = self.incr_indent();
                    writeln!(f)?;
                    indented.fmt_symbol(f, ret_val)?;
                }
            }
            SymbolKind::Export(exp) => {
                let indented = self.incr_indent();
                writeln!(f)?;

                if let Some(target) = exp.target {
                    indented.fmt_symbol(f, target)?;
                } else {
                    windentln!(indented, f, "MISSING TARGET")?;
                }
            }
            SymbolKind::Import(imp) => {
                if let Some(target) = imp.target {
                    write!(f, " => module{}", KeyDataFmt(target.data()))?;
                }

                writeln!(f)?;

                let indented = self.incr_indent();

                if let Some(import_expr) = imp.expr {
                    indented.fmt_symbol(f, import_expr)?;
                    writeln!(f)?;
                } else {
                    windentln!(indented, f, "MISSING IMPORT EXPR")?;
                }

                if let Some(alias) = imp.alias {
                    windentln!(indented, f, "alias")?;
                    indented.fmt_symbol(f, alias)?;
                }
            }
            SymbolKind::Switch(switch) => {
                let indented = self.incr_indent();

                writeln!(f)?;

                if let Some(target) = switch.target {
                    indented.fmt_symbol(f, target)?;
                } else {
                    windent!(indented, f, "MISSING SWITCH TARGET")?;
                }

                if !switch.arms.is_empty() {
                    writeln!(f)?;
                }

                let mut first = true;
                for arm in &switch.arms {
                    if !first {
                        writeln!(f)?;
                    }
                    first = false;

                    windentln!(indented, f, "match")?;
                    match arm.pat_expr {
                        Some(c) => {
                            indented.fmt_symbol(f, c)?;
                            writeln!(f)?;
                        }
                        None => windentln!(indented, f, "MISSING PATTERN")?,
                    }

                    if let Some(cond) = arm.condition_expr {
                        windentln!(indented, f, "if")?;
                        indented.fmt_symbol(f, cond)?;
                        writeln!(f)?;
                    }

                    windentln!(indented, f, "then")?;
                    match arm.pat_expr {
                        Some(c) => {
                            indented.fmt_symbol(f, c)?;
                        }
                        None => windent!(indented, f, "MISSING ARM VALUE")?,
                    }
                }
            }
            SymbolKind::Try(t) => {
                write!(f, " ")?;

                self.fmt_scope(f, t.try_scope)?;
                write!(f, " catch ")?;
                self.fmt_scope(f, t.catch_scope)?;
            }
            SymbolKind::Throw(t) => {
                writeln!(f)?;

                let indented = self.incr_indent();

                if let Some(target) = t.expr {
                    indented.fmt_symbol(f, target)?;
                } else {
                    windent!(indented, f, "MISSING THROW VALUE")?;
                }
            }
            SymbolKind::Virtual(virt) => match virt {
                VirtualSymbol::Proxy(proxy) => {
                    write!(f, " => ${}", KeyDataFmt(proxy.target.data()))?;
                }
                VirtualSymbol::Module(m) => {
                    write!(f, " {} => module{}", m.name, KeyDataFmt(m.module.data()))?;
                }
            },
            SymbolKind::Continue(_)
            | SymbolKind::Discard(_)
            | SymbolKind::TypeDecl(_)
            | SymbolKind::Op(_) => {
                // TODO: add these as needed
            }
        }

        Ok(())
    }
}

#[must_use]
pub struct ModuleFmt<'h> {
    module: Module,
    f: HirFmt<'h>,
}

impl fmt::Display for ModuleFmt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.f.fmt_module(f, self.module)
    }
}

#[must_use]
pub struct ScopeFmt<'h> {
    scope: Scope,
    f: HirFmt<'h>,
}

impl fmt::Display for ScopeFmt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.f.fmt_scope(f, self.scope)
    }
}

#[must_use]
pub struct SymbolFmt<'h> {
    symbol: Symbol,
    f: HirFmt<'h>,
}

impl fmt::Display for SymbolFmt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.f.fmt_symbol(f, self.symbol)
    }
}

#[must_use]
pub struct SourceFmt<'h> {
    source: Source,
    f: HirFmt<'h>,
}

impl fmt::Display for SourceFmt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.f.fmt_source(f, self.source)
    }
}

struct KeyDataFmt(KeyData);

impl fmt::Display for KeyDataFmt {
    #[allow(clippy::cast_possible_truncation)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // null value
        if Symbol::default().data().as_ffi() == self.0.as_ffi() {
            return write!(f, "@NULL");
        }

        let value = self.0.as_ffi();
        let idx = value & 0xffff_ffff;
        let version = (value >> 32) | 1;
        '@'.fmt(f)?;
        (idx as u32).fmt(f)?;
        ':'.fmt(f)?;
        (version as u32).fmt(f)?;
        Ok(())
    }
}
