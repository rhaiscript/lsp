use crate::{
    eval::Value,
    symbol::{ReferenceTarget, SymbolKind},
    ty::{Array, Function, Object, TypeData},
    Hir, IndexMap, IndexSet, Symbol, TypeKind,
};

impl Hir {
    pub(crate) fn resolve_types_for_all_symbols(&mut self) {
        let symbols = self.symbols.keys().collect::<Vec<_>>();

        for symbol in symbols {
            self.resolve_type_for_symbol(symbol);
        }
    }

    // An intermediate collect is sometimes required to satisfy
    // the borrow-checker. We operate on multiple elements of the 
    // same slotmap. Even if we know that the operations are disjoint,
    // the borrow-checker does not, and we need to index into it again,
    // or sometimes collect intermediate values into a vec.
    // This makes some operations somewhat more inefficient, but at the same
    // time some subtle bugs are turned into panics instead.
    #[allow(clippy::needless_collect)]
    pub(crate) fn resolve_type_for_symbol(&mut self, symbol: Symbol) {
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
                    })
                };
            }
            SymbolKind::If(if_sym) => {
                let branch_symbols = if_sym
                    .branches
                    .iter()
                    .map(|(_, scope)| self.scopes.get(*scope).unwrap().symbols.last().copied())
                    .collect::<Vec<_>>();

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
                        (decl.name.clone(), sym_data.ty)
                    })
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
                    self.symbols.get(last_expr).unwrap().ty
                } else {
                    self.builtin_types.unknown
                };

                self.symbols.get_mut(symbol).unwrap().ty = self.types.insert(TypeData {
                    source,
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
                        (decl.name.clone(), sym_data.ty)
                    })
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
                    self.symbols.get(last_expr).unwrap().ty
                } else {
                    self.builtin_types.unknown
                };

                self.symbols.get_mut(symbol).unwrap().ty = self.types.insert(TypeData {
                    source,
                    kind: TypeKind::Fn(Function {
                        is_closure: false,
                        params,
                        ret,
                    }),
                });
            }
            SymbolKind::Call(call) => {
                if let Some(lhs) = call.lhs {
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
                let types = arr.values.clone();

                let mut types = types
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
                    })
                };

                let arr_ty = self.types.insert(TypeData {
                    source,
                    kind: TypeKind::Array(Array { items }),
                });

                self.symbols.get_mut(symbol).unwrap().ty = arr_ty;
            }
            SymbolKind::Object(o) => {
                let fields = o
                    .fields
                    .iter()
                    .filter_map(|(name, field)| field.value.map(|val| (name.clone(), val)))
                    .collect::<Vec<_>>();

                let fields = fields
                    .into_iter()
                    .map(|(name, sym)| (name, self.symbols.get(sym).unwrap().ty))
                    .collect::<IndexMap<_, _>>();

                self.symbols.get_mut(symbol).unwrap().ty = self.types.insert(TypeData {
                    source,
                    kind: TypeKind::Object(Object { fields }),
                });
            }

            SymbolKind::Path(p) => {
                if let Some(&path_sym) = p.segments.last() {
                    self.symbols.get_mut(symbol).unwrap().ty =
                        self.symbols.get(path_sym).unwrap().ty;
                }
            }
            SymbolKind::Throw(_)
            | SymbolKind::Break(_)
            | SymbolKind::Continue(_)
            | SymbolKind::Return(_)
            | SymbolKind::Virtual(_)
            | SymbolKind::Discard(_)
            | SymbolKind::Op(_)
            | SymbolKind::Try(_) => {
                sym_data.ty = self.builtin_types.never;
            }
            SymbolKind::Import(_)
            | SymbolKind::Export(_)
            | SymbolKind::For(_)
            | SymbolKind::Loop(_)
            | SymbolKind::While(_) => {
                sym_data.ty = self.builtin_types.void;
            }
            SymbolKind::Unary(_) | SymbolKind::Binary(_) => {
                // TODO
            }
        }
    }
}
