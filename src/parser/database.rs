use anyhow::{bail, Context, Result};

use crate::model::database::Database;
use crate::parser::pages::parse_first_page;
use crate::parser::primitives::Cursor;

pub fn parse_database(bytes: &[u8]) -> Result<Database> {
    if bytes.len() < 100 {
        bail!("Invalid SQLite database: less than 100 bytes")
    }
    let first_100 = bytes.get(..100).unwrap();
    println!("{:?}", first_100);
    let mut cursor = Cursor {
        data: bytes,
        position: 0,
    };

    let first_page = parse_first_page(&mut cursor).context("Failed to parse first page")?;

    Ok(Database {
        first_page,
        pages: Vec::new(),
    })
}
