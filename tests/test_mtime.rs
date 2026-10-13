//! 测试文件修改时间（mtime）检测

use std::path::PathBuf;
use std::time::SystemTime;

fn format_system_time(time: &SystemTime) -> String {
    let duration = time.duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let secs = duration.as_secs();

    // 转换为可读格式 (UTC)
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    // 简单日期计算（从 1970-01-01 开始）
    let mut year = 1970;
    let mut remaining_days = days as i64;

    // 考虑闰年
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let mut month = 1;
    let days_in_month = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    for &dim in &days_in_month {
        if remaining_days < dim {
            break;
        }
        remaining_days -= dim;
        month += 1;
    }

    let day = remaining_days + 1;

    format!(
        "{}-{:02}-{:02} {:02}:{:02}:{:02} UTC (Unix: {})",
        year, month, day, hours, minutes, seconds, secs
    )
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[tokio::test]
async fn test_mtime_detection() {
    let cert_path = PathBuf::from("certs/server.crt");
    println!("文件: {}\n", cert_path.display());
    let x = std::fs::metadata(&cert_path).and_then(|m| m.modified()).unwrap();
    println!("time SystemTime (Debug): {:?}", x);
    println!("time 格式化: {}", format_system_time(&x));
}
