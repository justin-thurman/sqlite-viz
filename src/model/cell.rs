use serde::Serialize;

#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct BTreeTableLeafCell {
    pub payload_size_bytes: i64,
    pub rowid: i64,
    pub payload_size_on_page_bytes: u64,
    pub first_overflow_page_num: Option<u32>,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct BTreeTableInteriorCell {
    pub left_child_ptr: u32,
    pub key: i64,
}
