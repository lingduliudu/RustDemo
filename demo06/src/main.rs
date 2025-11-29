mod test_box;
mod test_fs;
mod test_hashmap;
use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataStoreError {
    #[error("数据流异常")]
    Disconnect(#[from] std::io::Error),
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
    #[error("unknown data store error")]
    Unknown,
    #[error("任意错误")]
    AnyError(#[from] anyhow::Error),
}

fn test_open_fail() -> Result<()> {
    let y = std::fs::read("dsfsdf.t")?;
    println!("{:?}", y);
    Ok(())
}

fn main() {
    //test_hashmap::test01();
    //test_box::test04();
    // test_fs::test02();
    test_open_fail().unwrap();
    println!("结束了吗");
}
