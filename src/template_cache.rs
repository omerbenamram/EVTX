use crate::evtx_parser::ReadSeek;
use crate::binxml::tokens::read_template_definition;
use crate::err::{self, Result};

use crate::model::deserialized::BinXMLTemplateDefinition;
use crate::Offset;
pub use byteorder::{LittleEndian, ReadBytesExt};
use snafu::ResultExt;
use std::collections::HashMap;
use std::io::{Cursor, Seek, SeekFrom};

pub type CachedTemplate<'chunk> = BinXMLTemplateDefinition<'chunk>;

#[derive(Debug, Default)]
pub struct TemplateCache<'chunk>(HashMap<Offset, CachedTemplate<'chunk>>);

impl<'chunk> TemplateCache<'chunk> {
    pub fn new() -> Self {
        TemplateCache(HashMap::new())
    }

    pub fn populate(data: &'chunk [u8], offsets: &[Offset]) -> Result<Self> {
        let mut cache = HashMap::new();
        let mut cursor = Cursor::new(data);

        for offset in offsets.iter().filter(|&&offset| offset > 0) {
            cursor
                .seek(SeekFrom::Start(u64::from(*offset)))
                .context(err::IO)?;

            let definition = read_template_definition(&mut cursor, None)?;
            cache.insert(*offset, definition);
        }

        Ok(TemplateCache(cache))
    }

    pub fn get_template<'a: 'chunk>(&'a self, offset: Offset) -> Option<&'a CachedTemplate<'a>> {
        self.0.get(&offset)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
