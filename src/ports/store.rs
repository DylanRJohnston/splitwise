use anyhow::Result;

pub trait Set {
    fn add(&self, key: i64) -> Result<()>;
    fn has(&self, key: i64) -> Result<bool>;
    fn batch_add(&self, key: Vec<i64>) -> Result<()>;
}
