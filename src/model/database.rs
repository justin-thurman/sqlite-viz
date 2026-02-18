use super::page::Page;

pub struct Database{
    pub database_header: DatabaseHeader,
    pub pages: Vec<Page>,
}

pub struct DatabaseHeader{
    pub page_size: u16,
}