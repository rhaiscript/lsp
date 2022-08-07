use crate::{
    eval::Value,
    hir::BuiltinTypes,
    symbol::{ReferenceTarget, SymbolKind},
    ty::{Array, Function, Object, Type, TypeData},
    HashSet, Hir, IndexMap, IndexSet, Symbol, TypeKind,
};
use slotmap::SlotMap;

impl Hir {
    pub(crate) fn resolve_types_for_all_symbols(&mut self) {
        let symbols = self.symbols.keys().collect::<Vec<_>>();

        let mut seen = HashSet::with_capacity(symbols.len());
        for symbol in symbols {
            self.resolve_type_for_symbol(&mut seen, symbol);
        }
    }

    pub(crate) fn resolve_type_aliases(&mut self) {
        let symbols = self.symbols.keys().collect::<Vec<_>>();

        let mut to_remove = HashSet::with_capacity(symbols.len());

        for symbol in symbols {
            let visible_types: Vec<_> = self
                .visible_symbols_from_symbol(symbol)
                .filter_map(|sym| {
                    if let Some(decl) = self[sym].kind.as_type_decl() {
                        if let TypeKind::Alias(name, ty) = &self.types.get(decl.ty).unwrap().kind {
                            Some((name.clone(), *ty))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            let symbol_data = self.symbols.get_mut(symbol).unwrap();

            match &mut symbol_data.kind {
                SymbolKind::Fn(sym) => {
                    resolve_and_replace(
                        &mut self.types,
                        self.builtin_types,
                        &mut sym.ret_ty,
                        &mut to_remove,
                        &visible_types,
                    );
                }
                SymbolKind::Op(sym) => {
                    resolve_and_replace(
                        &mut self.types,
                        self.builtin_types,
                        &mut sym.lhs_ty,
                        &mut to_remove,
                        &visible_types,
                    );

                    if let Some(rhs_ty) = &mut sym.rhs_ty {
                        resolve_and_replace(
                            &mut self.types,
                            self.builtin_types,
                            rhs_ty,
                            &mut to_remove,
                            &visible_types,
                        );
                    }

                    resolve_and_replace(
                        &mut self.types,
                        self.builtin_types,
                        &mut sym.ret_ty,
                        &mut to_remove,
                        &visible_types,
                    );
                }
                SymbolKind::Decl(sym) => {
                    if let Some(ty) = &mut sym.ty_decl {
                        resolve_and_replace(
                            &mut self.types,
                            self.builtin_types,
                            ty,
                            &mut to_remove,
                            &visible_types,
                        );
                    }
                }
                _ => {}
            }

            resolve_and_replace(
                &mut self.types,
                self.builtin_types,
                &mut symbol_data.ty,
                &mut to_remove,
                &visible_types,
            );
        }

        for ty in to_remove {
            self.remove_type(ty);
        }
    }

    /// Resolve and set the type for a symbol.
    ///
    /// Due to references and type-inference this function might
    /// be called recursively internally.
    ///
    /// In order to avoid excessive operations and infinite recursions
    /// or loops, we track which symbol was processed already.
    ///
    // An intermediate collect is sometimes required to satisfy
    // the borrow-checker. We operate on multiple elements of the
    // same slotmap. Even if we know that the operations are disjoint,
    // the borrow-checker does not, and we need to index into it again,
    // or sometimes collect intermediate values into a vec.
    // This makes some operations somewhat more inefficient, but at the same
    // time some subtle bugs are turned into panics instead.
    #[allow(clippy::needless_collect)]
    pub(crate) fn resolve_type_for_symbol(&mut self, seen: &mut HashSet<Symbol>, symbol: Symbol) {
        if seen.contains(&symbol) {
            return;
        }
        seen.insert(symbol);

        let sym_data = self.symbols.get_mut(symbol).unwrap();
        let source = sym_data.source;

        #[allow(clippy::match_same_arms)]
        match &sym_data.kind {
            SymbolKind::Lit(lit) => {
                sym_data.ty = match &lit.value {
                    Value::Int(_) => self.builtin_types.int,
                    Value::Float(_) => self.builtin_types.float,
                    Value::Bool(_) => self.builtin_types.bool,
                    Value::String(_) => self.builtin_types.string,
                    Value::Char(_) => self.builtin_types.char,
                    Value::Unknown => self.builtin_types.unknown,
                }
            }
            SymbolKind::Reference(r) => match r.target {
                Some(ReferenceTarget::Symbol(target_sym)) => {
                    self.resolve_type_for_symbol(seen, target_sym);
                    let target_sym_data = self.symbols.get(target_sym).unwrap();
                    self.symbols.get_mut(symbol).unwrap().ty = target_sym_data.ty;
                }
                Some(ReferenceTarget::Module(_)) => {
                    sym_data.ty = self.builtin_types.module;
                }
                None => sym_data.ty = self.builtin_types.unknown,
            },
            SymbolKind::Decl(decl) => {
                let ty = if let Some(ty) = decl.ty_decl {
                    ty
                } else if let Some(val) = decl.value {
                    self.resolve_type_for_symbol(seen, val);
                    self.symbols.get(val).unwrap().ty
                } else {
                    self.builtin_types.unknown
                };

                self.symbols.get_mut(symbol).unwrap().ty = ty;
            }
            SymbolKind::Block(block) => {
                if let Some(last_symbol) = self
                    .scopes
                    .get(block.scope)
                    .unwrap()
                    .symbols
                    .last()
                    .copied()
                {
                    self.resolve_type_for_symbol(seen, last_symbol);
                    self.symbols.get_mut(symbol).unwrap().ty =
                        self.symbols.get(last_symbol).unwrap().ty;
                }
            }
            SymbolKind::Switch(switch) => {
                let mut switch_types = IndexSet::default();
                let switch_arm_exprs = switch
                    .arms
                    .iter()
                    .filter_map(|arm| arm.value_expr)
                    .collect::<Vec<_>>();
                for arm_expr in switch_arm_exprs {
                    self.resolve_type_for_symbol(seen, arm_expr);
                    switch_types.insert(self.symbols.get(arm_expr).unwrap().ty);
                }
                self.symbols.get_mut(symbol).unwrap().ty = if switch_types.is_empty() {
                    self.builtin_types.void
                } else if switch_types.len() == 1 {
                    switch_types.pop().unwrap()
                } else {
                    self.types.insert(TypeData {
                        source,
                        kind: TypeKind::Union(switch_types),
                        protected: false,
                    })
                };
            }
            SymbolKind::If(if_sym) => {
                let branch_symbols = if_sym
                    .branches
                    .iter()
                    .map(|(_, scope)| self.scopes.get(*scope).unwrap().symbols.last().copied())
                    .collect::<Vec<_>>();

                for branch_sym in branch_symbols.iter().filter_map(|&s| s) {
                    self.resolve_type_for_symbol(seen, branch_sym);
                }

                let mut branch_types = branch_symbols
                    .into_iter()
                    .map(|sym| match sym {
                        Some(last_branch_sym) => self.symbols.get(last_branch_sym).unwrap().ty,
                        None => self.builtin_types.void,
                    })
                    .collect::<IndexSet<_>>();

                self.symbols.get_mut(symbol).unwrap().ty = if branch_types.is_empty() {
                    self.builtin_types.void
                } else if branch_types.len() == 1 {
                    branch_types.pop().unwrap()
                } else {
                    self.types.insert(TypeData {
                        source,
                        kind: TypeKind::Union(branch_types),
                        protected: false,
                    })
                };
            }
            SymbolKind::Fn(f) => {
                let scope = f.scope;
                let is_def = f.is_def;

                let ret_ty = if is_def && f.ret_ty == self.builtin_types.unknown {
                    self.builtin_types.void
                } else {
                    f.ret_ty
                };

                let params = self
                    .scopes
                    .get(f.scope)
                    .unwrap()
                    .symbols
                    .iter()
                    .copied()
                    .take_while(|&sym| self.symbols.get(sym).unwrap().is_param())
                    .map(|sym| {
                        let sym_data = self.symbols.get(sym).unwrap();
                        let decl = sym_data.kind.as_decl().unwrap();
                        (decl.name.clone(), sym)
                    })
                    .collect::<Vec<_>>();

                for (_, param) in &params {
                    self.resolve_type_for_symbol(seen, *param);
                }

                let params = params
                    .into_iter()
                    .map(|(name, sym)| (name, self.symbols.get(sym).unwrap().ty))
                    .collect::<Vec<_>>();

                let ret = if is_def {
                    ret_ty
                } else if let Some(last_expr) = self
                    .scopes
                    .get(scope)
                    .unwrap()
                    .symbols
                    .iter()
                    .copied()
                    .find(|&sym| !self.symbols.get(sym).unwrap().is_param())
                {
                    self.resolve_type_for_symbol(seen, last_expr);
                    self.symbols.get(last_expr).unwrap().ty
                } else {
                    self.builtin_types.unknown
                };

                self.symbols.get_mut(symbol).unwrap().ty = self.types.insert(TypeData {
                    source,
                    protected: false,
                    kind: TypeKind::Fn(Function {
                        is_closure: false,
                        params,
                        ret,
                    }),
                });
            }
            SymbolKind::Closure(f) => {
                let scope = f.scope;

                let params = self
                    .scopes
                    .get(f.scope)
                    .unwrap()
                    .symbols
                    .iter()
                    .copied()
                    .take_while(|&sym| self.symbols.get(sym).unwrap().is_param())
                    .map(|sym| {
                        let sym_data = self.symbols.get(sym).unwrap();
                        let decl = sym_data.kind.as_decl().unwrap();
                        (decl.name.clone(), sym)
                    })
                    .collect::<Vec<_>>();

                for (_, param) in &params {
                    self.resolve_type_for_symbol(seen, *param);
                }

                let params = params
                    .into_iter()
                    .map(|(name, sym)| (name, self.symbols.get(sym).unwrap().ty))
                    .collect::<Vec<_>>();

                let ret = if let Some(last_expr) = self
                    .scopes
                    .get(scope)
                    .unwrap()
                    .symbols
                    .iter()
                    .copied()
                    .find(|&sym| !self.symbols.get(sym).unwrap().is_param())
                {
                    self.resolve_type_for_symbol(seen, last_expr);
                    self.symbols.get(last_expr).unwrap().ty
                } else {
                    self.builtin_types.unknown
                };

                self.symbols.get_mut(symbol).unwrap().ty = self.types.insert(TypeData {
                    source,
                    protected: false,
                    kind: TypeKind::Fn(Function {
                        is_closure: false,
                        params,
                        ret,
                    }),
                });
            }
            SymbolKind::Call(call) => {
                if let Some(lhs) = call.lhs {
                    self.resolve_type_for_symbol(seen, lhs);
                    let ty_data = self.types.get(self.symbols.get(lhs).unwrap().ty).unwrap();

                    let ty = if let Some(ty_fn) = ty_data.kind.as_fn() {
                        ty_fn.ret
                    } else {
                        self.builtin_types.unknown
                    };

                    self.symbols.get_mut(symbol).unwrap().ty = ty;
                }
            }
            SymbolKind::Index(idx) => {
                if let Some(base) = idx.base {
                    self.resolve_type_for_symbol(seen, base);
                    let ty_data = self.types.get(self.symbols.get(base).unwrap().ty).unwrap();

                    let ty = if let Some(arr) = ty_data.kind.as_array() {
                        arr.items
                    } else {
                        self.builtin_types.unknown
                    };

                    self.symbols.get_mut(symbol).unwrap().ty = ty;
                }
            }
            SymbolKind::Array(arr) => {
                let elems = arr.values.clone();

                for elem in &elems {
                    self.resolve_type_for_symbol(seen, *elem);
                }

                let mut types = elems
                    .into_iter()
                    .map(|sym| self.symbols.get(sym).unwrap().ty)
                    .collect::<IndexSet<_>>();

                let items = if types.is_empty() {
                    self.builtin_types.void
                } else if types.len() == 1 {
                    types.pop().unwrap()
                } else {
                    self.types.insert(TypeData {
                        source,
                        kind: TypeKind::Union(types),
                        protected: false,
                    })
                };

                let arr_ty = self.types.insert(TypeData {
                    source,
                    kind: TypeKind::Array(Array { items }),
                    protected: false,
                });

                self.symbols.get_mut(symbol).unwrap().ty = arr_ty;
            }
            SymbolKind::Object(o) => {
                let fields = o
                    .fields
                    .iter()
                    .filter_map(|(name, field)| field.value.map(|val| (name.clone(), val)))
                    .collect::<Vec<_>>();

                for (_, field) in &fields {
                    self.resolve_type_for_symbol(seen, *field);
                }

                let fields = fields
                    .into_iter()
                    .map(|(name, sym)| (name, self.symbols.get(sym).unwrap().ty))
                    .collect::<IndexMap<_, _>>();

                self.symbols.get_mut(symbol).unwrap().ty = self.types.insert(TypeData {
                    source,
                    kind: TypeKind::Object(Object { fields }),
                    protected: false,
                });
            }

            SymbolKind::Path(p) => {
                if let Some(&path_sym) = p.segments.last() {
                    self.resolve_type_for_symbol(seen, path_sym);
                    self.symbols.get_mut(symbol).unwrap().ty =
                        self.symbols.get(path_sym).unwrap().ty;
                }
            }
            SymbolKind::Binary(b) => {
                let (lhs, rhs) = (b.lhs, b.rhs);
                let lookup_text = b.lookup_text.clone();

                let ty = if b.is_field_access() {
                    lhs.map(|lhs| {
                        self.resolve_type_for_symbol(seen, lhs);
                        lhs
                    })
                    .and_then(|lhs| self[self[lhs].ty].kind.as_object())
                    .and_then(|object| Some((object, rhs.and_then(|rhs| self[rhs].name(self))?)))
                    .and_then(|(object, field_name)| object.fields.get(field_name))
                    .copied()
                } else {
                    match (lhs, rhs) {
                        (Some(lhs), Some(rhs)) => {
                            self.resolve_type_for_symbol(seen, lhs);
                            self.resolve_type_for_symbol(seen, rhs);

                            let lhs_ty = self[lhs].ty;
                            let rhs_ty = self[rhs].ty;

                            // (lhs, rhs, ret)
                            let mut op_types = self
                                .symbols
                                .keys()
                                .filter_map(|sym| {
                                    if let Some(op) = self[sym].kind.as_op() {
                                        if op.name == lookup_text
                                            && op.lhs_ty.is(self, lhs_ty, false)
                                            && op.rhs_ty?.is(self, rhs_ty, false)
                                        {
                                            Some((op.lhs_ty, op.rhs_ty?, op.ret_ty))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>();

                            let exact_types = op_types
                                .iter()
                                .find(|(op_lhs, op_rhs, _)| {
                                    op_lhs.is(self, lhs_ty, true) && op_rhs.is(self, rhs_ty, true)
                                })
                                .copied();

                            exact_types.or_else(|| op_types.pop()).map(|(.., ty)| ty)
                        }
                        _ => None,
                    }
                };

                if let Some(ty) = ty {
                    self.symbols.get_mut(symbol).unwrap().ty = ty;
                } else {
                    self.symbols.get_mut(symbol).unwrap().ty = self.builtin_types.unknown;
                }
            }
            SymbolKind::Unary(u) => {
                let lookup_text = u.lookup_text.clone();
                if let Some(rhs_ty) = u.rhs.map(|rhs| self[rhs].ty) {
                    // (lhs/rhs, ret)
                    let mut op_types = self
                        .symbols
                        .keys()
                        .filter_map(|sym| {
                            if let Some(op) = self[sym].kind.as_op() {
                                if op.name == lookup_text
                                    && op.rhs_ty.is_none()
                                    && op.lhs_ty.is(self, rhs_ty, false)
                                {
                                    Some((op.lhs_ty, op.ret_ty))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    let exact_types = op_types
                        .iter()
                        .find(|(op_lhs, _)| op_lhs.is(self, rhs_ty, true))
                        .copied();

                    let ty = exact_types
                        .map(|(_, ret)| ret)
                        .or_else(|| op_types.pop().map(|(_, ret)| ret))
                        .or(Some(rhs_ty))
                        .unwrap_or(self.builtin_types.unknown);

                    self.symbols.get_mut(symbol).unwrap().ty = ty;
                }
            }
            SymbolKind::Throw(_)
            | SymbolKind::Break(_)
            | SymbolKind::Continue(_)
            | SymbolKind::Return(_)
            | SymbolKind::Virtual(_)
            | SymbolKind::Discard(_)
            | SymbolKind::Op(_)
            | SymbolKind::Try(_)
            | SymbolKind::TypeDecl(_) => {
                sym_data.ty = self.builtin_types.never;
            }
            SymbolKind::Import(_)
            | SymbolKind::Export(_)
            | SymbolKind::For(_)
            | SymbolKind::Loop(_)
            | SymbolKind::While(_) => {
                sym_data.ty = self.builtin_types.void;
            }
        }
    }
}

fn resolve_and_replace(
    types: &mut SlotMap<Type, TypeData>,
    builtin_types: BuiltinTypes,
    ty: &mut Type,
    to_remove: &mut HashSet<Type>,
    visible_types: &[(String, Type)],
) {
    if let TypeKind::Unresolved(r) = &types.get(*ty).unwrap().kind {
        match r.trim() {
            "module" => {
                to_remove.insert(*ty);
                *ty = builtin_types.module;
            }
            "int" => {
                to_remove.insert(*ty);
                *ty = builtin_types.int;
            }
            "float" => {
                to_remove.insert(*ty);
                *ty = builtin_types.float;
            }
            "bool" => {
                to_remove.insert(*ty);
                *ty = builtin_types.bool;
            }
            "char" => {
                to_remove.insert(*ty);
                *ty = builtin_types.char;
            }
            "String" => {
                to_remove.insert(*ty);
                *ty = builtin_types.string;
            }
            "timestamp" => {
                to_remove.insert(*ty);
                *ty = builtin_types.timestamp;
            }
            "void" | "()" => {
                to_remove.insert(*ty);
                *ty = builtin_types.void;
            }
            "?" => {
                to_remove.insert(*ty);
                *ty = builtin_types.unknown;
            }
            "!" => {
                to_remove.insert(*ty);
                *ty = builtin_types.never;
            }
            name => {
                if let Some((name, alias_ty)) =
                    visible_types.iter().find(|(def_name, _)| def_name == name)
                {
                    // to_remove.insert(*ty);
                    let original_ty_source = types.get(*ty).unwrap().source;

                    *ty = types.insert(TypeData {
                        source: original_ty_source,
                        kind: TypeKind::Alias(name.clone(), *alias_ty),
                        protected: false,
                    });
                }
            }
        }
    }
}
