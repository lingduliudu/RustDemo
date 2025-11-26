use chrono::{DateTime, FixedOffset, Utc};
use std::env;
use std::fs;
use std::io;
use std::time::SystemTime;
fn get_exe_age_in_seconds() -> Result<u64, io::Error> {
    let exe_path = env::current_exe()?;
    let metadata = fs::metadata(&exe_path)?;
    let creation_time = match metadata.created() {
        Ok(time) => time,
        Err(_) => {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "无法获取文件的创建时间",
            ));
        }
    };
    // 4. 获取当前时间
    let created_datetime: DateTime<Utc> = creation_time.into();
    let timezone_cst = FixedOffset::east_opt(8 * 3600)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "无效的时区偏移量"))?;
    let created_cst = created_datetime.with_timezone(&timezone_cst);
    println!(
        "文件创建时间 (UTC): {}",
        created_cst.format("%Y-%m-%d %H:%M:%S %:z")
    );
    let current_time = SystemTime::now();
    let elapsed_duration = current_time
        .duration_since(creation_time)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("计算时间差失败：{:?}", e)))?;
    Ok(elapsed_duration.as_secs())
}

pub fn is_expire() {
    let x: u64 = get_exe_age_in_seconds().unwrap();
    println!("相差秒数:{}", x);
    // 1个月
    if x > 3600 * 24 * 30 {
        std::process::exit(0);
    }
}
