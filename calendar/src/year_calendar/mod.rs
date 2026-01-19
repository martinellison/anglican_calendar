/*! Implements a calendar for a specific year, as derived from a
[crate::perpetual::calendar::Calendar]. */
extern crate askama;
use crate::perpetual::{CalendarError, HolydayClass, HolydayRef, MainAttribute};
use askama::Template;
use chrono::{Datelike, Days, Duration, NaiveDate, Weekday};
use serde::Serialize;
use std::{cmp::Ordering, collections::HashMap};
/// a value or an error code
type Result<T> = std::result::Result<T, CalendarError>;
mod year;
use getset::{CopyGetters, Getters, MutGetters};
pub use year::Year;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests2;
pub mod year_calendar;
/** whether a [YearHolyday] will be dropped. */
#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub enum DropStatus {
    Keep,
    Drop(DropReason),
}
impl std::fmt::Display for DropStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Keep => std::result::Result::Ok(()),
            Self::Drop(reason) => write!(f, "{}", reason),
        }
    }
}
/** reason for dropping a [YearHolyday] */
#[derive(Debug, Eq, PartialEq, Clone, strum::Display, Serialize)]
pub enum DropReason {
    Easter,
    Clash,
    Sunday,
    Cutoff,
    // Other,
}
#[derive(Debug, Eq, PartialEq, Clone, Getters, MutGetters, CopyGetters)]
/** A YearHolyday is an holyday in the calendar for a specific year
([year_calendar::YearCalendar]) e.g. in the 2019 calendar of the Anglican Church of
Hong Kong, Easter Sunday was 21 April and Matteo Ricci was 11 May. */
#[derive(Serialize)]
pub struct YearHolyday {
    holyday: HolydayRef,
    #[getset(get_copy = "pub(crate)")]
    date: NaiveDate,
    is_eve: bool,
    drop_status: DropStatus,
    advice: Vec<String>,
}
impl YearHolyday {
    /** Create a [YearHolyday] from an [Holyday](crate::perpetual::holyday::Holyday) given the [Year]
    data. */
    pub fn from_holyday(holyday: &HolydayRef, year: &Year, is_eve: bool) -> Result<Self> {
        let orig_date = year.date_cal_to_date(&holyday.date_cal())?;
        let date = if is_eve {
            Self::eve(orig_date)
        } else {
            orig_date
        };
        let mut advice = vec![];
        if is_eve {
            advice.push(format!("eve of {}", &holyday.title()));
        }
        Ok(Self {
            holyday: holyday.clone(),
            date,
            advice,
            drop_status: DropStatus::Keep,
            is_eve,
        })
    }

    /** `eve` calculates the date of the eve/vigil of a holy day */
    pub fn eve(date: NaiveDate) -> NaiveDate {
        if date.weekday() == Weekday::Mon {
            date - Days::new(2)
        } else {
            date - Days::new(1)
        }
    }

    /** Change the date of a [YearHolyday] by a specified [Duration] */
    pub fn change_date_by(&mut self, cd: Duration) {
        self.date += cd;
        self.add_advice(format!(
            "{} ({:?}/{:?}) changed to {} (modified by {:?})",
            self.holyday.title(),
            self.holyday.class(),
            self.holyday.transfer(),
            &self.date,
            cd
        ));
    }

    /** set the date of a [YearHolyday] to the specified date */
    pub fn change_date_to(&mut self, date: NaiveDate) {
        self.date = date;
        self.add_advice(format!(
            "{} ({:?}/{:?}) changed to {}",
            self.holyday.title(),
            self.holyday.class(),
            self.holyday.transfer(),
            &self.date
        ));
    }

    /** the display colour for this holy day, from the CSS3 set of
    colour names, see [colours](https://www.w3.org/TR/css-color-3) */
    pub fn colour(&self, year: &Year, advice: &mut Vec<String>) -> String {
        /* "If the Collect, Readings, etc. on a Lesser Festival are
        those of the saint, then either red (for a martyr) or white is
        used; " TODO how does the user choose? code*/
        let is_martyr = self.holyday.main().contains(&MainAttribute::Martyr);
        if is_martyr {
            advice.push(format!(
                "{} ({}) is Martyr so colour is red",
                self.holyday.title(),
                self.holyday.class(),
            ));
            return "red".to_string();
        }
        match self.holyday.class() {
            // HolydayClass::Principal |  WHY NOT (Annunciation, Trinity) ? not Ash Wed, Palm Sun,
            // Good Fri, Easter Eve, Pentecost all red
            HolydayClass::CorpusChristi | HolydayClass::Festival | HolydayClass::LesserFestival => {
                advice.push(format!(
                    "{} ({}) has colour white",
                    self.holyday.title(),
                    self.holyday.class(),
                ));
                "white".to_string()
            },
            _ => year.season_colour(self.date, advice).to_string(),
        }
    }

    /** `add_advice` adds some advice (for report) */
    pub fn add_advice<S>(&mut self, advice: S)
    where
        S: Into<String>,
    {
        self.advice.push(advice.into());
    }
}
impl Ord for YearHolyday {
    fn cmp(&self, other: &Self) -> Ordering { self.date.cmp(&other.date) }
}

impl PartialOrd for YearHolyday {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}
/// the template for a report (for merging with the data)
#[derive(Template)]
#[template(path = "report.html")]
struct ReportTemplate {
    province: String,
    year: i32,
    dates: Vec<ReportDate>,
}
/// the template for a wall calendar (for merging with the data)
#[derive(Template)]
#[template(path = "wall_calendar.html")]
struct WallTemplate {
    province: String,
    year: i32,
    months: Vec<WallMonth>,
}
impl WallTemplate {
    /** Create a new WallTemplate */
    pub fn new(province: String, year: i32, holydays: &Vec<ReportDate>) -> Self {
        let mut months = vec![];
        for m in 1..=12 {
            months.push(WallMonth::new(year, m));
        }
        for day in holydays {
            let m = day.date.month();
            // let loc = months[m as usize].day_locs.get(&day.date).unwrap();
            // months[m as usize].weeks[loc.w as usize].days[loc.wd as usize] =
            //     WallDay::Holy(day.clone());
            months[m as usize - 1].make_holy(day);
        }
        Self {
            province,
            year,
            months,
        }
    }
}
struct DayLoc {
    w: u8,
    wd: u8,
}
/** A `WallMonth`  is the data for a month of a wall calendar */
pub struct WallMonth {
    weeks: Vec<WallWeek>,
    name: String,
    day_locs: HashMap<NaiveDate, DayLoc>,
}
impl WallMonth {
    /** Create a new WallMonth */
    pub fn new(y: i32, m: u32) -> Self {
        let name = NaiveDate::from_ymd_opt(y, m, 1)
            .unwrap()
            .format("%B")
            .to_string();
        let mut weeks = vec![];
        let mut day_locs = HashMap::new();
        // find the Sunday of the first week of the month (typically in the previous
        // month)
        let start = NaiveDate::from_ymd_opt(y, m, 1)
            .unwrap()
            .week(Weekday::Sun)
            .first_day();
        // debug!("month calendar starts on {start}");
        let mut current_week = -1;
        for md in 1..=31 {
            if let Some(day) = NaiveDate::from_ymd_opt(y, m, md) {
                // find which week of the month this day is in
                let w = day.week(Weekday::Sun);
                let wfd = w.first_day();
                let wdiff = wfd - start;
                let week_num = wdiff.num_weeks();
                // debug!(
                //     "day {y}-{m:02}-{md:02} is {day}, week starts {wfd} so week num
                // {week_num}, \      current week is {current_week}"
                // );
                if week_num > current_week {
                    assert_eq!(week_num, current_week + 1);
                    weeks.push(WallWeek::new(y, m, week_num.try_into().unwrap()));
                    current_week = week_num;
                    assert_eq!(current_week, (weeks.len() as i64) - 1);
                }
                let wd = day.weekday().number_from_sunday() - 1;
                weeks[current_week as usize].days[wd as usize] = WallDay::Unholy(day);
                day_locs.insert(
                    day,
                    DayLoc {
                        w: current_week.try_into().unwrap(),
                        wd: wd.try_into().unwrap(),
                    },
                );
            }
        }
        Self {
            weeks,
            name,
            day_locs,
        }
    }

    /** `make_holy` makes a day holy */
    pub fn make_holy(&mut self, day: &ReportDate) {
        let loc = self
            .day_locs
            .get(&day.date)
            .unwrap_or_else(|| panic!("should have day {} in {}", &day.date, &self.name));
        self.weeks[loc.w as usize].days[loc.wd as usize] = WallDay::Holy(day.clone());
    }
}
/** A `WallWeek`  is the data for a week in a wall calendar */
#[derive(Debug, Clone, Default)]
pub struct WallWeek {
    days: [WallDay; 7],
}
impl WallWeek {
    fn new(_y: i32, _m: u32, _w: u8) -> Self { Self::default() }

    /** `has_day` week has at lease one day */
    pub fn has_day(&self) -> bool { self.days.iter().any(|day| day.is_day()) }
}
/** A `WallDay`  is the data for a day in a wall calendar */
#[derive(Debug, Clone, Default)]
pub enum WallDay {
    Holy(ReportDate),
    Unholy(NaiveDate),
    #[default]
    NotDay,
}
impl WallDay {}
impl WallDay {
    pub fn new() -> Self { Default::default() }

    pub fn is_day(&self) -> bool { !matches!(self, Self::NotDay) }
}
/// All the data for a calendar date including holy days
#[derive(Debug, Clone)]
struct ReportDate {
    date: NaiveDate, // is constructed and used in report
    date_form: String,
    holydays: Vec<ReportHolyday>,
}
/// A [Holyday](crate::perpetual::holyday::Holyday) as it appears in a report
#[derive(Debug, Clone)]
struct ReportHolyday {
    title: String,
    colour: String,
    // description: String,
    class_format: String,
    other: Vec<String>,
    refs_format: Vec<(String, String)>,
    death: String,
    attrs: Vec<String>,
    is_eve: bool,
    drop_status: DropStatus,
    advice: Vec<String>,
}
impl ReportHolyday {
    /** `descr` is a heading that describes this day */
    pub fn descr(&self) -> String {
        format!(
            "{}{}",
            if self.is_eve { "Eve of " } else { "" },
            &self.title,
        )
    }
}

/*

Copyright ©2019-2024 Martin Ellison.  This program is free software: you
can redistribute it and/or modify it under the terms of the GNU
General Public License as published by the Free Software Foundation,
either version 3 of the License, or (at your option) any later
version.

This program is distributed in the hope that it will be useful, but
WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see
[licenses](https://www.gnu.org/licenses/). */
