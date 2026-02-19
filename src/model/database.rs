use super::page::Page;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Database {
    pub database_header: DatabaseHeader,
    pub pages: Vec<Page>,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct DatabaseHeader {
    pub header_string: String,
    pub page_size: u16,
    pub file_format_write_version: FileFormatVersion,
    pub file_format_read_version: FileFormatVersion,
    pub reserved_bytes: u8,
    pub max_embedded_payload_fraction: u8,
    pub min_embedded_payload_fraction: u8,
    pub leaf_payload_fraction: u8,
    pub file_change_counter: u32,
    pub database_size_pages: u32,
    pub first_freelist_trunk_page: u32,
    pub total_freelist_pages: u32,
    pub schema_cookie: u32,
    pub schema_format_number: u32,
    pub default_page_cache_size: u32,
    pub largest_root_btree_page: u32,
    pub database_text_encoding: TextEncoding,
    pub user_version: u32,
    pub incremental_vacuum_mode: bool,
    pub application_id: u32,
    pub version_valid_for: u32,
    pub sqlite_version_number: u32,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
pub enum FileFormatVersion {
    Legacy,
    Wal,
    Other(u8), // for future proofing
}

impl From<u8> for FileFormatVersion {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Legacy,
            2 => Self::Wal,
            other => Self::Other(other),
        }
    }
}

#[derive(Debug, Serialize, Eq, PartialEq)]
pub enum TextEncoding {
    Utf8,
    Utf16le,
    Utf16be,
    Other(u32),
}

impl From<u32> for TextEncoding {
    fn from(value: u32) -> Self {
        match value {
            1 => Self::Utf8,
            2 => Self::Utf16le,
            3 => Self::Utf16be,
            other => Self::Other(other),
        }
    }
}
