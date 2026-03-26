use std::env;
use std::fs;
use std::io::{self, Write};

fn parse_timestamp(ts: &str) -> i64 {
    let parts: Vec<&str> = ts.split([':', ',']).collect();
    let h: i64 = parts[0].parse().unwrap();
    let m: i64 = parts[1].parse().unwrap();
    let s: i64 = parts[2].parse().unwrap();
    let ms: i64 = parts[3].parse().unwrap();

    h * 3600_000 + m * 60_000 + s * 1000 + ms
}

fn format_timestamp(mut ms: i64) -> String {
    if ms < 0 {
        ms = 0;
    }

    let h = ms / 3600_000;
    ms %= 3600_000;
    let m = ms / 60_000;
    ms %= 60_000;
    let s = ms / 1000;
    let ms = ms % 1000;

    format!("{:02}:{:02}:{:02},{:03}", h, m, s, ms)
}

fn shift_line(line: &str, offset: i64) -> String {
    if line.contains("-->") {
        let parts: Vec<&str> = line.split("-->").collect();
        let start = parse_timestamp(parts[0].trim());
        let end = parse_timestamp(parts[1].trim());

        let new_start = format_timestamp(start + offset);
        let new_end = format_timestamp(end + offset);

        format!("{} --> {}", new_start, new_end)
    } else {
        line.to_string()
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        eprintln!(
            "Usage: {} <input.srt> <output.srt> <offset_ms>",
            args[0]
        );
        std::process::exit(1);
    }

    let input_file = &args[1];
    let output_file = &args[2];
    let offset: i64 = args[3].parse().expect("Invalid offset");

    let content = fs::read_to_string(input_file)?;

    let mut result = String::with_capacity(content.len());

    for line in content.lines() {
        let new_line = shift_line(line, offset);
        result.push_str(&new_line);
        result.push('\n');
    }

    fs::write(output_file, result)?;

    println!("Done. Output written to {}", output_file);

    Ok(())
}
