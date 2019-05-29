extern crate anglican_calendar;
use anglican_calendar::calendar;
use ansi_term::Colour::*;
use ron::ser::to_string_pretty;
use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::io::{BufReader, BufWriter};
fn main() {
    let mods = calendar::EdMods {
        holydays: vec![
            calendar::HolydayMod {
                class: Some(calendar::HolydayClass::Festival),
                tag: "the baptism of christ".to_string(),
                date_cal: Some(calendar::DateCal::Next {
                    date: Box::new(calendar::DateCal::Fixed { month: 1, day: 6 }),
                    day_of_week: calendar::OrderableDayOfWeek::from(chrono::Weekday::Sun),
                }),
                ..calendar::HolydayMod::default()
            },
            calendar::HolydayMod {
                class: Some(calendar::HolydayClass::Principal),
                tag: "ash wednesday".to_string(),
                date_cal: Some(calendar::DateCal::After {
                    date: Box::new(calendar::DateCal::Easter),
                    rel: -46,
                }),
                ..calendar::HolydayMod::default()
            },
        ],
    };
    let ofn = "/tmp/exper.txt";
    let of = File::create(ofn)
        .map_err(calendar::CalendarError::from_error)
        .unwrap();
    let mut bw = BufWriter::new(of);
    let s = to_string_pretty(&mods, ron::ser::PrettyConfig::default()).unwrap();
    let _ = bw.write(s.as_bytes()).unwrap();
    bw.flush().unwrap()
}
