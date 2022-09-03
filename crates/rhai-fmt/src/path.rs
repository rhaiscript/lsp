use std::io::{self, Write};

use rhai_rowan::ast::Path;

use crate::{algorithm::Formatter, util::ScopedStatic};

impl<S: Write> Formatter<S> {
    pub(crate) fn fmt_path(&mut self, path: Path) -> io::Result<()> {
        let mut first = true;
        for segment in path.segments() {
            if !first {
                self.word("::")?;
            }
            first = false;

            self.word(segment.static_text())?;
        }
        Ok(())
    }
}
