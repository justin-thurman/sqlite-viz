use anyhow::{bail, Context, Result};

use crate::model::database::FileFormatVersion;
use crate::model::database::TextEncoding;
use crate::model::database::{Database, DatabaseHeader};
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

    let database_header = parse_database_header(&mut cursor).context("Invalid database header")?;
    assert_eq!(
        cursor.position, 100,
        "Cursor position not correct after reading database header"
    );

    Ok(Database {
        pages: Vec::new(),
        database_header,
    })
}

fn parse_database_header(cursor: &mut Cursor) -> Result<DatabaseHeader> {
    let header_bytes = cursor.read_array::<16>();
    let header_string =
        str::from_utf8(&header_bytes).context("Failed to parse first 16 bytes to string")?;
    if header_string != "SQLite format 3\0" {
        bail!("Header string is invalid: {}", header_string)
    }
    let page_size = cursor.u16_from_be();
    if !cursor.data.len().is_multiple_of(page_size as usize) as usize != 0 {
        bail!("File length is not an exact multiple of page size")
    }

    let file_format_write_version: FileFormatVersion = cursor.u8_from_be().into();
    let file_format_read_version: FileFormatVersion = cursor.u8_from_be().into();

    let reserved_bytes = cursor.u8_from_be();

    let max_embedded_payload_fraction = cursor.u8_from_be();
    let min_embedded_payload_fraction = cursor.u8_from_be();
    let leaf_payload_fraction = cursor.u8_from_be();

    if max_embedded_payload_fraction != 64 {
        bail!(
            "Invalid max embedded payload fraction: {}",
            max_embedded_payload_fraction
        )
    }
    if min_embedded_payload_fraction != 32 {
        bail!(
            "Invalid min embedded payload fraction: {}",
            min_embedded_payload_fraction
        )
    }
    if leaf_payload_fraction != 32 {
        bail!("Invalid leaf payload fraction: {}", leaf_payload_fraction)
    }

    let file_change_counter = cursor.u32_from_be();
    let database_size_pages = cursor.u32_from_be();
    let first_freelist_trunk_page = cursor.u32_from_be();
    let total_freelist_pages = cursor.u32_from_be();
    let schema_cookie = cursor.u32_from_be();
    let schema_format_number = cursor.u32_from_be();
    let default_page_cache_size = cursor.u32_from_be();
    let largest_root_btree_page = cursor.u32_from_be();
    let database_text_encoding: TextEncoding = cursor.u32_from_be().into();
    let user_version = cursor.u32_from_be();
    let incremental_vacuum_mode = cursor.u32_from_be() != 0;
    let application_id = cursor.u32_from_be();

    let reserved_for_expansion = cursor.read_array::<20>();
    if reserved_for_expansion != [0u8; 20] {
        bail!("Bytes reserved for expansion must be zero")
    }

    let version_valid_for = cursor.u32_from_be();
    let sqlite_version_number = cursor.u32_from_be();

    assert_eq!(
        cursor.position, 100,
        "Cursor position not correct after reading database header"
    );

    Ok(DatabaseHeader {
        header_string: "SQLite format 3".into(), // dropping the null byte
        page_size,
        file_format_write_version,
        file_format_read_version,
        reserved_bytes,
        max_embedded_payload_fraction,
        min_embedded_payload_fraction,
        leaf_payload_fraction,
        file_change_counter,
        database_size_pages,
        first_freelist_trunk_page,
        total_freelist_pages,
        schema_cookie,
        schema_format_number,
        default_page_cache_size,
        largest_root_btree_page,
        database_text_encoding,
        user_version,
        incremental_vacuum_mode,
        application_id,
        version_valid_for,
        sqlite_version_number,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_database_header_success() {
        let data =
            std::fs::read("tests/fixtures/test.db").expect("Failed to read database fixture");

        let mut cursor = Cursor {
            data: &data,
            position: 0,
        };
        let expected = DatabaseHeader {
            header_string: "SQLite format 3".into(),
            page_size: 4096,
            file_format_write_version: FileFormatVersion::Legacy,
            file_format_read_version: FileFormatVersion::Legacy,
            reserved_bytes: 12,
            max_embedded_payload_fraction: 64,
            min_embedded_payload_fraction: 32,
            leaf_payload_fraction: 32,
            file_change_counter: 11,
            database_size_pages: 3,
            first_freelist_trunk_page: 0,
            total_freelist_pages: 0,
            schema_cookie: 2,
            schema_format_number: 4,
            default_page_cache_size: 0,
            largest_root_btree_page: 0,
            database_text_encoding: TextEncoding::Utf8,
            user_version: 0,
            incremental_vacuum_mode: false,
            application_id: 0,
            version_valid_for: 11,
            sqlite_version_number: 3043002,
        };

        let result = parse_database_header(&mut cursor).expect("Parsing header should succeed");

        assert_eq!(expected, result);
    }
}
