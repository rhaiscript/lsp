use crate::{
    symbol::{ReferenceTarget, SymbolKind},
    Hir,
};

impl Hir {
    pub fn clear_references(&mut self) {
        let ref_symbols = self.symbols.iter_mut();

        for (_, sym_data) in ref_symbols {
            match &mut sym_data.kind {
                SymbolKind::Fn(f) => f.references.clear(),
                SymbolKind::Op(f) => f.references.clear(),
                SymbolKind::Decl(d) => d.references.clear(),
                SymbolKind::Reference(r) => r.target = None,
                _ => {}
            }
        }
    }

    pub fn resolve_references(&mut self) {
        self.clear_references();

        let self_ptr = self as *mut Hir;

        let ref_symbols = self
            .symbols
            .iter_mut()
            .filter_map(|(s, d)| match &mut d.kind {
                SymbolKind::Reference(r) if !r.part_of_path => Some((s, r)),
                _ => None,
            });

        for (symbol, ref_kind) in ref_symbols {
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
                            ref_kind.target = Some(ReferenceTarget::Symbol(vis_symbol));
                            break;
                        }
                        SymbolKind::Decl(target) => {
                            target.references.insert(symbol);
                            ref_kind.target = Some(ReferenceTarget::Symbol(vis_symbol));
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}