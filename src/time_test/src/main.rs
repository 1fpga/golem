use std::time::Instant;
use time::{OffsetDateTime, UtcOffset};

fn main() {
    let now = Instant::now();
    println!("1 {:?}", now);

    let now = OffsetDateTime::now_utc();
    let now_local = OffsetDateTime::now_local().unwrap();

    println!("3 {}", now);
    println!("4 {}", now_local);

    let unix_time_seconds = now.unix_timestamp();
    let offset = OffsetDateTime::from_unix_timestamp(unix_time_seconds)
        .ok()
        .and_then(|t| UtcOffset::local_offset_at(t).ok())
        .map_or(0, UtcOffset::whole_seconds);
    eprintln!("5 {}", offset);
}
