use anyhow::{bail, Result};

use crate::model::database::{Database, DatabaseHeader};
use crate::parser::primitives::Cursor;

pub fn parse_database(bytes: &[u8]) -> Result<Database> {
    if bytes.len() < 100 {
        bail!("Invalid SQLite database: less than 100 bytes")
    }
    // FIXME: Just going to jump straight to page size for initial POC skeleton
    let mut cursor = Cursor {
        data: bytes,
        position: 16,
    };
    let page_size = cursor.u16_from_be();
    let database_header = DatabaseHeader { page_size };
    Ok(Database {
        pages: Vec::new(),
        database_header,
    })
}
