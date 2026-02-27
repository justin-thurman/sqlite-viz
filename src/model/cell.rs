use serde::Serialize;

#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct BTreeTableLeafCell {
    pub payload_size_bytes: u64,
    pub rowid: u64,
    pub payload_size_on_page_bytes: u16,
    pub first_overflow_page_num: Option<u32>,
}
