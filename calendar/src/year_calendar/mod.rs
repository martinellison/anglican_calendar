/*! Implements a calendar for a specific year, as derived from a
[crate::calendar::calendar::Calendar](Calendar). */
extern crate askama;
use crate::perpetual::{CalendarError, HolydayClass, HolydayRef, MainAttribute, SeasonColour};
use askama::Template;
use chrono::{Duration, NaiveDate};
use std::cmp::Ordering;
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
#[derive(Debug, Eq, PartialEq, Clone)]
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
#[derive(Debug, Eq, PartialEq, Clone, strum::Display)]
pub enum DropReason {
    Easter,
    Clash,
    Sunday,
    Cutoff,
    // Other,
}
#[derive(Debug, Eq, PartialEq, Clone, Getters, MutGetters, CopyGetters)]
/** A YearHolyday is an holyday in the calendar for a specific year
([YearCalendar]) e.g. in the 2019 calendar of the Anglican Church of
Hong Kong, Easter Sunday was 21 April and Matteo Ricci was 11 May. */
pub struct YearHolyday {
    holyday: HolydayRef,
    #[getset(get_copy = "pub(crate)")]
    date: NaiveDate,
    drop_status: DropStatus,
    advice: Vec<String>,
}
impl YearHolyday {
    /** Create a [YearHolyday] from an [calendar::Holyday] given the [Year]
     * data. */
    pub fn from_holyday(holyday: &HolydayRef, year: &Year) -> Result<Self> {
        Ok(Self {
            holyday: holyday.clone(),
            date: year.date_cal_to_date(&holyday.date_cal())?,
            advice: vec![],
            drop_status: DropStatus::Keep,
        })
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
            // HolydayClass::Principal |
          HolydayClass::CorpusChristi |
            // |
            HolydayClass::Festival | HolydayClass::LesserFestival => {
                advice.push(format!(
                    "{} ({}) has colour white",
                    self.holyday.title(),
                    self.holyday.class(),
                ));
                "white".to_string()
            },
            _ => match year.season_colour(self.date, advice) {
                SeasonColour::White => "white".to_string(),
                SeasonColour::Red => "red".to_string(),
                SeasonColour::Purple => "purple".to_string(),
                SeasonColour::Green => "green".to_string(),
            },
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
/// All the data for a calendar date including holy days
#[derive(Debug, Clone)]
struct ReportDate {
    //  date: NaiveDate, // is constructed and used in report
    date_form: String,
    holydays: Vec<ReportHolyday>,
    // colour_a: String,
    // colour_b: String,
}
/// A [Holyday](calendar::Calendar) as it appears in a report
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
    drop_status: DropStatus,
    advice: Vec<String>,
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
