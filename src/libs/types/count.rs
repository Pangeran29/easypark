use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SqlxCount {
    pub data: Option<i64>
}