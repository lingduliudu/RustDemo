use futures::executor::block_on;
use futures::join;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() {
    let x = test();
    block_on(x);
}

async fn test() {
    let x = download_01();
    let y = download_02();
    join!(x, y);
}

async fn download_01() {
    sleep(Duration::from_secs(4)).await;
    println!("01 完毕!");
}

async fn download_02() {
    sleep(Duration::from_secs(7)).await;
    println!("02 完毕!");
}
