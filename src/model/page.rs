use crate::model::database::DatabaseHeader;
use crate::model::page::BTreePageType::{IndexInterior, IndexLeaf, TableInterior, TableLeaf};
use serde::Serialize;
use std::convert::TryFrom;
use thiserror::Error;

#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct Page {}

#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct FirstPage {
    pub database_header: DatabaseHeader,
    pub page_header: PageHeader,
    pub cell_ptr_array: Vec<u16>,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct PageHeader {
    pub page_type: BTreePageType, // TODO: currently treating all pages as B-tree; will decide how to split up later
    pub first_freeblock_start: u16, // 0 -> no freeblocks
    pub num_cells: u16,
    pub cell_content_area_start: u16, // 0 -> 65536
    pub num_fragmented_free_bytes: u8,
    pub rightmost_ptr: Option<u32>, // present only for interior b-tree pages
}

#[derive(Debug, Serialize, Eq, PartialEq)]
pub enum BTreePageType {
    IndexInterior,
    IndexLeaf,
    TableInterior,
    TableLeaf,
}

impl BTreePageType {
    pub fn is_interior(&self) -> bool {
        self == &TableInterior || self == &IndexInterior
    }
}

#[derive(Debug, Error)]
pub enum BTreePageTypeError {
    #[error("Invalid B-tree page type: {0}")]
    InvalidPageType(u8),
}

impl TryFrom<u8> for BTreePageType {
    type Error = BTreePageTypeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            2 => Ok(IndexInterior),
            5 => Ok(TableInterior),
            10 => Ok(IndexLeaf),
            13 => Ok(TableLeaf),
            _ => Err(BTreePageTypeError::InvalidPageType(value)),
        }
    }
}
