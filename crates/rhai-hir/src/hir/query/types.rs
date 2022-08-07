use crate::{hir::BuiltinTypes, Hir};

impl Hir {
    #[must_use]
    #[inline]
    pub const fn builtin_types(&self) -> BuiltinTypes {
        self.builtin_types
    }
}
