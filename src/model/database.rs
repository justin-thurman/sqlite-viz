use super::page::Page;
use serde::Serialize;

#[derive(Serialize)]
pub struct Database {
    pub database_header: DatabaseHeader,
    pub pages: Vec<Page>,
}

#[derive(Serialize)]
pub struct DatabaseHeader {
    pub page_size: u16,
}
