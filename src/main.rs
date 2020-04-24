use std::str::FromStr;

use chrono::{
    format::{self, StrftimeItems},
    DateTime, Local, NaiveDateTime, Utc,
};
use clap::{App, Arg};

// 1900
const LOWER_BOUND: i64 = -2208988800;
// 2500
const UPPER_BOUND: i64 = 16725225600;

fn parse_i64(s: &str) -> Result<DateTime<Utc>, ()> {
    match i64::from_str(s) {
        Ok(ts) if ts < UPPER_BOUND && ts > LOWER_BOUND => Ok(DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp(ts, 0),
            Utc,
        )),
        Ok(ts) => Ok(DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp(ts / 1000, (ts % 1000) as u32),
            Utc,
        )),
        Err(_) => Err(()),
    }
}

fn parse_f64(s: &str) -> Result<DateTime<Utc>, ()> {
    match f64::from_str(s) {
        Ok(ts) => {
            let ts = if ts < UPPER_BOUND as f64 && ts > LOWER_BOUND as f64 {
                (ts * 1000.).round() as i64
            } else {
                ts.round() as i64
            };
            Ok(DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp(ts / 1000, (ts % 1000) as u32),
                Utc,
            ))
        }
        Err(_) => Err(()),
    }
}

fn parse_dt_str(fmt: &'static str) -> impl Fn(&str) -> Result<DateTime<Utc>, ()> {
    move |s| {
        let mut p = format::Parsed::new();
        format::parse(&mut p, s, StrftimeItems::new(fmt)).map_err(|_| ())?;
        if p.hour_mod_12.is_none() {
            p.set_hour(0).unwrap();
        }
        if p.minute.is_none() {
            p.set_minute(0).unwrap();
        }
        if p.day.is_none() {
            p.set_day(1).unwrap();
        }
        if p.month.is_none() {
            p.set_month(1).unwrap();
        }
        let dt = DateTime::<Utc>::from_utc(
            p.to_naive_date()
                .map_err(|_| ())?
                .and_time(p.to_naive_time().map_err(|_| ())?),
            Utc,
        );
        if dt.timestamp() > LOWER_BOUND && dt.timestamp() < UPPER_BOUND {
            Ok(dt)
        } else {
            Err(())
        }
    }
}

fn parse(s: &str) -> Result<DateTime<Utc>, ()> {
    // See if it's basic unix time
    for p in [
        &parse_dt_str("%Y") as &dyn Fn(&str) -> Result<_, _>,
        &parse_dt_str("%Y%m"),
        &parse_dt_str("%Y%m%d"),
        &parse_dt_str("%Y%m%d%H"),
        &parse_dt_str("%Y%m%d%H%M"),
        &|ss| {
            DateTime::parse_from_rfc2822(ss)
                .map(|t| t.with_timezone(&Utc))
                .map_err(|_| ())
        },
        &|ss| {
            DateTime::parse_from_rfc3339(ss)
                .map(|t| t.with_timezone(&Utc))
                .map_err(|_| ())
        },
        &parse_dt_str("%Y-%m-%dT%H:%M:%S"),
        &parse_dt_str("%Y-%m-%dT%H:%M"),
        &parse_i64,
        &parse_f64,
    ]
    .iter()
    {
        if let Ok(t) = p(s) {
            return Ok(t);
        }
    }
    Err(())
}

fn main() {
    let app = App::new("time-cli")
        .version("0.1.0")
        .author("Robert Ying <rbtying@aeturnalus.com>")
        .about("Command-line utility for parsing timestamps")
        .arg(
            Arg::with_name("DATETIME")
                .help("A time or date, e.g. a Unix timestamp")
                .required(false)
                .index(1),
        );
    let matches = app.get_matches();

    let now = Utc::now();

    let utc_ts = match matches.value_of("DATETIME") {
        Some(s) => match parse(s) {
            Ok(ts) => ts,
            Err(()) => {
                eprintln!("Unable to parse timestamp {}", s);
                eprintln!("{}", matches.usage());
                return;
            }
        },
        None => now,
    };

    println!("{:20}{:.03}", "Unix time:", utc_ts.timestamp());
    println!(
        "{:20}{:.03}",
        "Unix time (float):",
        utc_ts.timestamp_millis() as f64 / 1000.
    );
    println!("{:20}{}", "Unix time (ms):", utc_ts.timestamp_millis());
    println!("");

    if utc_ts < now {
        let elapsed = now - utc_ts;
        println!("{:20}{}", "Seconds since", elapsed.num_seconds());
        if elapsed.num_hours() > 0 {
            println!("{:20}{}", "Hours since", elapsed.num_hours());
        }
        if elapsed.num_days() > 0 {
            println!("{:20}{}", "Days since", elapsed.num_days());
        }
        println!("");
    } else if utc_ts > now {
        let until = utc_ts - now;
        println!("{:20}{}", "Seconds until", until.num_seconds());
        if until.num_hours() > 0 {
            println!("{:20}{}", "Hours until", until.num_hours());
        }
        if until.num_days() > 0 {
            println!("{:20}{}", "Days until", until.num_days());
        }
        println!("");
    }

    println!("{:20}{}", "RFC2822 UTC:", utc_ts.to_rfc2822());
    println!("{:20}{}", "RFC3339 UTC:", utc_ts.to_rfc3339());
    println!("{:20}{}", "YMD UTC:", utc_ts.format("%Y%m%d"));
    println!("{:20}{}", "YMDH UTC:", utc_ts.format("%Y%m%d%H"));
    println!("");

    let local_ts = utc_ts.with_timezone(&Local);
    println!("{:20}{}", "RFC2822 Local:", local_ts.to_rfc2822());
    println!("{:20}{}", "RFC3339 Local:", local_ts.to_rfc3339());
}
