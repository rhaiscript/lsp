use crate::{
    symbol::{ReferenceTarget, SymbolKind},
    ty::{Array, Function, Object},
    Module, Symbol, Type,
};

impl Module {
    pub(crate) fn resolve_references(&mut self) {
        let self_ptr = self as *mut Module;

        let ref_symbols = self
            .symbols
            .iter_mut()
            .filter_map(|(s, d)| match &mut d.kind {
                SymbolKind::Reference(r) => Some((s, r)),
                _ => None,
            });

        for (symbol, ref_kind) in ref_symbols {
            ref_kind.target = None;

            // safety: This is safe because we only operate
            //  on separate elements (declarations and refs)
            //  and we don't touch the map itself.
            //
            //  Without this unsafe block we wouldn't be able to
            //  modify values in place.
            unsafe {
                for vis_symbol in (&*self_ptr).visible_symbols_from_symbol(symbol) {
                    let vis_symbol_data = (*self_ptr).symbols.get_unchecked_mut(vis_symbol);
                    if let Some(n) = vis_symbol_data.name() {
                        if n != ref_kind.name {
                            continue;
                        }
                    }

                    match &mut vis_symbol_data.kind {
                        SymbolKind::Fn(target) => {
                            target.references.insert(symbol);
                        }
                        SymbolKind::Decl(target) => {
                            target.references.insert(symbol);
                        }
                        _ => unreachable!(),
                    }

                    ref_kind.target = Some(ReferenceTarget::Symbol(vis_symbol));
                    break;
                }
            }
        }
    }
}

impl Module {
    pub(crate) fn infer_types(&mut self) {
        let decl_symbols = self
            .symbols
            .iter()
            .filter_map(|s| {
                if matches!(s.1.kind, SymbolKind::Decl(_) | SymbolKind::Fn(_)) {
                    Some(s.0)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for symbol in decl_symbols {
            let mut ty = self.infer_symbol_type_for(symbol);

            match &self[symbol].kind {
                SymbolKind::Fn(_) => ty = self.infer_symbol_type_for(symbol),
                SymbolKind::Decl(dec) => {
                    if let Some(value_symbol) = dec.value {
                        ty = self.infer_symbol_type_for(value_symbol);
                    }
                }
                _ => {}
            }

            match &mut self.symbol_unchecked_mut(symbol).kind {
                SymbolKind::Fn(f) => f.ty = ty,
                SymbolKind::Decl(d) => d.ty = ty,
                _ => {}
            }
        }
    }

    fn infer_symbol_type_for(&self, symbol: Symbol) -> Type {
        match &self[symbol].kind {
            SymbolKind::Lit(lit) => lit.ty.clone(),
            SymbolKind::Array(arr) => {
                let known_items = arr
                    .values
                    .iter()
                    .map(|s| self.infer_symbol_type_for(*s))
                    .collect::<Vec<_>>();
                Type::Array(Array {
                    item_types: Box::new(known_items.iter().fold(Type::Unknown, |t1, t2| t1 + t2)),
                    known_items,
                })
            }
            SymbolKind::Object(obj) => Type::Object(Object {
                fields: obj
                    .fields
                    .iter()
                    .map(|(name, field)| {
                        (
                            name.clone(),
                            field
                                .value
                                .map_or(Type::Unknown, |v| self.infer_symbol_type_for(v)),
                        )
                    })
                    .collect(),
            }),
            SymbolKind::Fn(f) => Type::Fn(Function {
                is_closure: false,
                params: f
                    .params(self)
                    .map(|param_symbol| {
                        let param = &self[param_symbol].kind.as_decl().unwrap();
                        (param.name.clone(), Type::Unknown)
                    })
                    .collect(),
                ret: Box::new(Type::Unknown),
            }),
            SymbolKind::Closure(f) => Type::Fn(Function {
                is_closure: false,
                params: f
                    .params(self)
                    .map(|param_symbol| {
                        let param = &self[param_symbol].kind.as_decl().unwrap();
                        (param.name.clone(), Type::Unknown)
                    })
                    .collect(),
                ret: Box::new(Type::Unknown),
            }),
            SymbolKind::Reference(r) => match r.target {
                Some(ReferenceTarget::Symbol(target_symbol)) => match &self[target_symbol].kind {
                    SymbolKind::Fn(_) => self.infer_symbol_type_for(symbol),
                    SymbolKind::Decl(dec) => dec.value.map_or(Type::Unknown, |value_symbol| {
                        self.infer_symbol_type_for(value_symbol)
                    }),
                    _ => Type::Unknown,
                },
                _ => Type::Unknown,
            },
            SymbolKind::Decl(_) => Type::Void,
            _ => Type::Unknown,
        }
    }
}
