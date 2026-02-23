use crate::model::database::DatabaseHeader;
use serde::Serialize;

#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct Page {}

#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct FirstPage {
    pub database_header: DatabaseHeader,
}
