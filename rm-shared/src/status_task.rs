#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusTask {
    Add(String),
    Delete(String),
}
