/*! Implements a calendar for a specific year, as derived from a
[calendar::Calendar](Calendar). */
extern crate askama;
use crate::calendar::{self, CalendarError};
use ansi_term::Colour::*;
use askama::Template;
use chrono::{Datelike, Duration, NaiveDate};
use icalendar::*;
use std::{cmp::Ordering, collections::HashMap, io::Write};
/// a value or an error code
type Result<T> = std::result::Result<T, calendar::CalendarError>;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests2;

#[derive(Debug, Eq, PartialEq, Clone)]
/** A YearCalendar is a calendar for a specific year for a specific
church e.g. the 2019 calendar of the Anglican Church of Hong Kong. */
pub struct YearCalendar {
    province: calendar::Province,
    year: Year,
    holydays_by_date: HashMap<NaiveDate, Vec<YearHolyday>>,
}
impl YearCalendar {
    /** Create a YearCalendar from a [Calendar] given the year. */
    pub fn from_calendar(calendar: &calendar::Calendar, year: i32, verbose: bool) -> Result<Self> {
        let y = Year::new(year);
        let mut ycal = Self {
            year: y.clone(),
            province: calendar.province,
            holydays_by_date: HashMap::new(),
        };
        for e in calendar.get_holydays() {
            let mut year_holyday = YearHolyday::from_holyday(&e, &ycal.year)?;
            // println!(
            //     "{}",
            //     Green.bold().paint(format!(
            //         "converting {} ({:?}) {}",
            //         e.title(),
            //         e.class(),
            //         year_holyday.colour(&y)
            //     ))
            // );
            ycal.add(&mut year_holyday, &y, verbose, true)?; // TODO program
                                                             // option keep_dropped
        }
        Ok(ycal)
    }

    /** Generate an iCalendar. Also generate the calendar updates to cancel the entries.

    See [RFC 5545](https://tools.ietf.org/html/rfc5545) and [RFC
    7986](https://tools.ietf.org/html/rfc7986) for details of the
    iCalendar format. */

    pub fn to_ical(&self, unique: &str) -> (icalendar::Calendar, icalendar::Calendar) {
        let mut ical = icalendar::Calendar::new();

        /* Not sure how to do this: add calendar properties here
        e.g. set REFRESH-INTERVAL to P4W i.e. refresh every 4
        weeks */

        let mut ical_del = icalendar::Calendar::new();
        println!("unique code for holydays is {}", unique);
        let mut ix = 0;
        let mut dates: Vec<&NaiveDate> = self.holydays_by_date.keys().collect();
        dates.sort();
        for date in dates {
            let year_holydays = &self.holydays_by_date[date];
            for year_holyday in year_holydays {
                let uid = format!("{}-{}", unique, ix);
                let mut e1a = icalendar::Event::new();
                let mut advice = vec![];
                let e = e1a
                    .summary(&year_holyday.holyday.title())
                    .description(&year_holyday.holyday.description())
                    .all_day(year_holyday.date)
                    .uid(&uid)
                    .append_property(icalendar::Property::new("TRANSP", "TRANSPARENT"))
                    .append_property(icalendar::Property::new("SEQUENCE", "0"))
                    .append_property(icalendar::Property::new(
                        "COLOR",
                        &year_holyday.colour(&self.year, &mut advice),
                    ))
                    .done();
                ical.push(e);
                let e_del = icalendar::Event::new()
                    .uid(&uid)
                    .append_property(icalendar::Property::new("STATUS", "CANCELLED"))
                    .append_property(icalendar::Property::new("SEQUENCE", "1"))
                    .done();
                ical_del.push(e_del);
                ix += 1;
            }
        }
        (ical, ical_del)
    }

    /** Write a human-readable report to a file. */
    pub fn write_report(&self, w: &mut dyn Write) -> Result<()> {
        let mut rt = ReportTemplate {
            dates: vec![],
            year: self.year.ad,
            province: self.province.to_string(),
        };
        let mut dates: Vec<&NaiveDate> = self.holydays_by_date.keys().collect();
        dates.sort();
        for date in dates {
            let mut advice = vec![];
            // let season_colour = self.year.season_colour(*date, &mut advice);
            let mut rd = ReportDate {
                // date: *date,
                date_form: date.format("%A %B %e").to_string(),
                holydays: vec![],
                // colour_a: colour.colour_a(),
                // colour_b: season_colour.colour_b(),
            };
            let year_holydays = &self.holydays_by_date[date];
            for year_holyday in year_holydays {
                let colour = year_holyday.colour(&self.year, &mut advice);
                let mut refs_format: Vec<(String, String)> = vec![];
                for r in &year_holyday.holyday.refs() {
                    let desc = r.description.trim();
                    if !desc.is_empty() {
                        refs_format.push((r.url(), desc.to_string()));
                    }
                }
                let mut rhd_advice = advice.clone();
                rhd_advice.extend_from_slice(&year_holyday.advice);
                let rhd = ReportHolyday {
                    title: year_holyday.holyday.title().clone(),
                    colour,
                    // description: year_holyday.holyday.description().clone(),
                    other: year_holyday.holyday.other().clone(),
                    refs_format,
                    class_format: year_holyday.holyday.class().to_string(),
                    death: year_holyday.holyday.death().to_string(),
                    attrs: year_holyday
                        .holyday
                        .main()
                        .iter()
                        .map(|attr| attr.to_string())
                        .collect(),
                    drop_status: year_holyday.drop_status.clone(),
                    advice: rhd_advice,
                };
                rd.holydays.push(rhd);
            }
            rt.dates.push(rd);
        }
        let r = rt.render().map_err(calendar::CalendarError::from_error)?;
        w.write_all(r.as_bytes())
            .map_err(calendar::CalendarError::from_error)
        //        Ok(())
    }

    /// add a holyday to a report
    fn add(
        &mut self,
        year_holyday: &mut YearHolyday,
        year: &Year,
        verbose: bool,
        keep_dropped: bool,
    ) -> Result<()> {
        if verbose {
            println!(
                "for {} adding {}",
                year_holyday.date,
                year_holyday.holyday.title()
            );
        }
        if let Some(day_holydays) = self.holydays_by_date.get_mut(&year_holyday.date) {
            Self::add_holyday_if_ok(day_holydays, year_holyday, year, keep_dropped)?;
        } else {
            if verbose {
                println!("new date {}", year_holyday.date);
            }
            let mut de = vec![];
            Self::add_holyday_if_ok(&mut de, year_holyday, year, keep_dropped)?;
            self.holydays_by_date.insert(year_holyday.date, de); // may insert
                                                                 // empty list,
                                                                 // is ok
        };
        Ok(())
    }

    /// add a holyday to a report if it is not dropped
    fn add_holyday_if_ok(
        day_holydays: &mut Vec<YearHolyday>,
        year_holyday: &mut YearHolyday,
        year: &Year,
        keep_dropped: bool,
    ) -> Result<()> {
        let ds = Self::fix_holyday_date_is_ok(day_holydays, year_holyday, year);
        year_holyday.drop_status = ds.clone();
        if let DropStatus::Drop(reason) = ds.clone() {
            let msg = format!(
                "{} dropped because: {reason}",
                &year_holyday.holyday.title()
            );
            println!("{}", Yellow.bold().paint(&msg));
            year_holyday.add_advice(msg);
        }
        if ds == DropStatus::Keep || keep_dropped {
            day_holydays.push(year_holyday.clone());
        } else if let DropStatus::Drop(r) = ds {
            println!(
                "{}",
                Yellow.bold().paint(format!(
                    "{} ({}) dropped because {:?}",
                    year_holyday.holyday.title(),
                    year_holyday.date,
                    r
                ))
            );
        }
        Ok(())
    }

    /**
     Tests if an holyday exists for the current year and, if necessary,
     transfers the holyday to another date, to avoid clashes with other
     holydays that might be on the same date.

    See [the rules](https://www.churchofengland.org/prayer-and-worship/worship-texts-and-resources/common-worship/prayer-and-worship/worship-texts-and-resources/common-worship/churchs-year/rules).

    The calculation assumes that 'saints days' means commemorations and lesser festivals.

    Note that some lesser festivals will be dropped altogether e.g. if
    they appear on a Sunday but commemorations appearing on the same day
    will not be dropped.
                 */
    pub fn fix_holyday_date_is_ok(
        day_holydays: &[YearHolyday],
        year_holyday: &mut YearHolyday,
        year: &Year,
    ) -> DropStatus {
        // calculate some dates and date ranges

        let is_sunday = year_holyday.date.weekday() == chrono::Weekday::Sun;
        if is_sunday {
            year_holyday.add_advice(format!(
                "{} ({}) is Sunday",
                year_holyday.holyday.title(),
                year_holyday.date
            ));
        }
        // let is_weekday = match year_holyday.date.weekday() {
        //     chrono::Weekday::Sat | chrono::Weekday::Sun => false,
        //     _ => true,
        // };
        let is_in_advent = year_holyday.date >= year.advent_next
            && year_holyday.date < NaiveDate::from_ymd_opt(year.ad, 12, 25).expect("invalid date");
        if is_in_advent {
            year_holyday.add_advice(format!(
                "{} ({}) is in advent",
                year_holyday.holyday.title(),
                year_holyday.date
            ));
        }
        let is_in_lent_or_eastertide =
            year_holyday.date >= year.ash_wednesday && year_holyday.date <= year.pentecost;
        if is_in_lent_or_eastertide {
            year_holyday.add_advice(format!(
                "{} ({}) is in Lent or Eastertide",
                year_holyday.holyday.title(),
                year_holyday.date
            ));
        }
        //        let is_in_holy_week = year_holyday.date >= year.palm_sunday &&
        // year_holyday.date < year.easter;
        let is_in_easter =
            year_holyday.date >= year.palm_sunday && year_holyday.date <= year.easter_sunday_2;
        if is_in_easter {
            year_holyday.add_advice(format!(
                "{} ({}) is in Easter",
                year_holyday.holyday.title(),
                year_holyday.date
            ));
        }

        let day_has_holyday = !day_holydays.is_empty();
        let mut clash_level = calendar::HolydayClass::NotAFestival;
        //      let mut multi_level = false;
        if day_has_holyday {
            for ce in day_holydays.iter() {
                let cel = ce.holyday.class();
                if cel > clash_level {
                    clash_level = cel;
                    //   multi_level = true;
                }
            }
            let clashes = day_holydays
                .iter()
                .map(|yh| yh.holyday.title())
                .collect::<Vec<String>>()
                .join(", ");
            year_holyday.add_advice(format!(
                "date clash: {} already has {} holydays: {clashes}",
                &year_holyday.date,
                day_holydays.len()
            ));
            // print!("{}", year_holyday.holyday.title());
            // println!();
        }

        let t = year_holyday.holyday.transfer();
        let c = year_holyday.holyday.class();
        let clash_higher = day_has_holyday && clash_level > c;
        if clash_higher {
            year_holyday.add_advice(format!(
                "{c} coincides with a higher holy day ({clash_level}), transfer is {t}"
            ));
        }
        match t {
            // TODO no 'saints days' in Easter Week
            calendar::TransferType::Normal => match c {
                calendar::HolydayClass::Commemoration => {
                    /* no transfer required */
                    // TODO should drop on Sunday?
                    if is_in_easter {
                        year_holyday.add_advice("commemoration in Easter");
                        DropStatus::Drop(DropReason::Easter)
                    } else if is_sunday {
                        // TODO this should depend on transfer type
                        year_holyday.add_advice("commemoration on Sunday");
                        DropStatus::Drop(DropReason::Sunday)
                    } else {
                        DropStatus::Keep
                    }
                },
                calendar::HolydayClass::LesserFestival => {
                    if is_in_easter {
                        year_holyday.add_advice("lesser festival in Easter");
                        DropStatus::Drop(DropReason::Easter)
                    } else if clash_higher {
                        year_holyday.add_advice("lesser festival clash");
                        DropStatus::Drop(DropReason::Clash)
                    } else if is_sunday {
                        year_holyday.add_advice("lesser festival on Sunday");
                        DropStatus::Drop(DropReason::Sunday)
                    } else {
                        DropStatus::Keep
                    }
                },
                calendar::HolydayClass::Festival | calendar::HolydayClass::CorpusChristi => {
                    if (is_sunday && (is_in_advent || is_in_lent_or_eastertide)) || clash_higher {
                        year_holyday.add_advice("Festival or Corpus Christi date changed");
                        year_holyday.change_date_by(Duration::days(1))
                    }
                    DropStatus::Keep
                },
                calendar::HolydayClass::Principal => {
                    // if day_has_holyday {
                    //     // Err(calendar::CalendarError::new(&format!(
                    //     //     "principal holyday {} may not be moved; reorder to start of
                    // holydays.",     //     year_holyday.holyday.borrow().title
                    //     // )))
                    //     DropStatus::Drop(DropReason::Clash) // ??
                    // } else {
                    DropStatus::Keep
                    // }
                },
                calendar::HolydayClass::Sunday => {
                    /* what about Annunciation?? */
                    assert!(is_sunday);
                    if clash_higher {
                        year_holyday.add_advice("Sunday clash");
                        DropStatus::Drop(DropReason::Clash)
                    } else {
                        DropStatus::Keep
                    }
                },
                calendar::HolydayClass::Unclassified => {
                    /* no transfer required? */
                    DropStatus::Keep
                },
                calendar::HolydayClass::NotAFestival => panic!("bad class"),
            },
            calendar::TransferType::Annunciation => {
                if is_sunday {
                    year_holyday.add_advice("Annunciation on Sunday");
                    year_holyday.change_date_by(Duration::days(1))
                }
                DropStatus::Keep
            },
            // calendar::TransferType::BaptismOfChrist => {
            //     Ok(true)
            // }
            calendar::TransferType::Joseph => {
                if is_in_easter {
                    // let abvm = NaiveDate::from_ymd_opt(year.ad, 3, 25).expect("invalid date");
                    let days = if year.annunciation >= year.palm_sunday
                        && year.annunciation <= year.easter_sunday_2
                    {
                        2
                    } else {
                        1
                    };
                    year_holyday.add_advice("date change");
                    year_holyday.change_date_to(year.easter_sunday_2 + Duration::days(days));
                }
                DropStatus::Keep
            },
            calendar::TransferType::George => {
                if is_in_easter {
                    year_holyday.add_advice("date change");
                    year_holyday.change_date_to(year.easter_sunday_2 + Duration::days(1))
                }
                DropStatus::Keep
            },
            calendar::TransferType::Mark => {
                if is_in_easter {
                    let sgd = year_holyday.date - Duration::days(2);
                    let days = if sgd >= year.palm_sunday && sgd <= year.easter_sunday_2 {
                        2
                    } else {
                        1
                    };
                    year_holyday.add_advice("date change");
                    year_holyday.change_date_to(year.easter_sunday_2 + Duration::days(days));
                }
                DropStatus::Keep
            },
            // calendar::TransferType::BeforeAdventNext => {
            //     if year_holyday.date >= year.advent_next {
            //         DropStatus::Drop(DropReason::Cutoff)
            //     } else {
            //         DropStatus::Keep
            //     }
            // },
            calendar::TransferType::DoNotTransfer => DropStatus::Keep,
        }
    }
}
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
#[derive(Debug, Eq, PartialEq, Clone)]
/** A YearHolyday is an holyday in the calendar for a specific year
([YearCalendar]) e.g. in the 2019 calendar of the Anglican Church of
Hong Kong, Easter Sunday was 21 April and Matteo Ricci was 11 May. */
pub struct YearHolyday {
    holyday: calendar::HolydayRef,
    date: NaiveDate,
    drop_status: DropStatus,
    advice: Vec<String>,
}
impl YearHolyday {
    /** Create a [YearHolyday] from an [calendar::Holyday] given the [Year]
     * data. */
    pub fn from_holyday(holyday: &calendar::HolydayRef, year: &Year) -> Result<Self> {
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
        let is_martyr = self
            .holyday
            .main()
            .contains(&calendar::MainAttribute::Martyr);
        if is_martyr {
            advice.push(format!(
                "{} ({}) is Martyr so colour is red",
                self.holyday.title(),
                self.holyday.class(),
            ));
            return "red".to_string();
        }
        match self.holyday.class() {
            // calendar::HolydayClass::Principal
            // | calendar::HolydayClass::CorpusChristi
            // |
            calendar::HolydayClass::Festival | calendar::HolydayClass::LesserFestival => {
                advice.push(format!(
                    "{} ({}) has colour white",
                    self.holyday.title(),
                    self.holyday.class(),
                ));
                "white".to_string()
            },
            _ => match year.season_colour(self.date, advice) {
                calendar::SeasonColour::White => "white".to_string(),
                calendar::SeasonColour::Red => "red".to_string(),
                calendar::SeasonColour::Purple => "purple".to_string(),
                calendar::SeasonColour::Green => "green".to_string(),
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
#[derive(Debug, Eq, PartialEq, Clone)]
/** A Year contains data for a specific year e.g. the date of Easter.

This includes some precalculated dates that are used

* as a basis for the dates of moveable holydays
* to determine whether a date in in a specified range e.g. within Easter

n.b. We are taking holy week from Palm Sunday to Holy Saturday
inclusive. */
pub struct Year {
    /** year AD/CE for the calendar. */
    pub ad: i32,
    /** date of Easter Sunday */
    pub easter: NaiveDate,
    /** date of previous Advent Sunday */
    pub advent_previous: NaiveDate,
    /** date of next Advent Sunday */
    pub advent_next: NaiveDate,
    /** date of  Ash Wednesday */
    pub ash_wednesday: NaiveDate,
    /** date of  Palm Sunday */
    pub palm_sunday: NaiveDate,
    /** date of the second Sunday in Easter */
    pub easter_sunday_2: NaiveDate,
    /** date of Pentecost */
    pub pentecost: NaiveDate,
    /** date of the Annunciation */
    pub annunciation: NaiveDate,
    /** date of Christmas */
    pub christmas_next: NaiveDate,
    /** date of Presentation */
    pub presentation: NaiveDate,
    /** date of All Saints' */
    pub all_saints: NaiveDate,
    // /** date of */  pub : NaiveDate,
}
impl Year {
    /** a year with calculated dates
     ```
    use anglican_calendar::year_calendar::Year;
    use chrono::NaiveDate;
    assert_eq!(
        Year {
            ad: 2020,
            easter: NaiveDate::from_ymd_opt(2020, 4, 12),
            advent_previous: NaiveDate::from_ymd(2019, 12, 1),
            advent_next: NaiveDate::from_ymd(2020, 11, 29),
            ash_wednesday: NaiveDate::from_ymd(2020, 2, 26),
            palm_sunday: NaiveDate::from_ymd(2020, 4, 5),
            easter_sunday_2: NaiveDate::from_ymd(2020, 4, 19),
            pentecost: NaiveDate::from_ymd(2020, 5, 31),
        },
        Year::new(2020)
    );
    assert_eq!(
        Year {
            ad: 2019,
            easter: NaiveDate::from_ymd(2019, 4, 21),
            advent_previous: NaiveDate::from_ymd(2018, 12, 2),
            advent_next: NaiveDate::from_ymd(2019, 12, 1),
            ash_wednesday: NaiveDate::from_ymd(2019, 3, 6),
            palm_sunday: NaiveDate::from_ymd(2019, 4, 14),
            easter_sunday_2: NaiveDate::from_ymd(2019, 4, 28),
            pentecost: NaiveDate::from_ymd(2019, 6, 9),
        },
        Year::new(2019)
    );
     ```
     */
    pub fn new(year: i32) -> Self {
        let easter = NaiveDate::from_ymd_opt(year, 3, 1).expect("invalid date")
            + Duration::days(i64::from(Self::computus(year) - 1));
        Self {
            ad: year,
            easter,
            advent_previous: Year::previous_inclusive(
                Year::next_inclusive(
                    NaiveDate::from_ymd_opt(year - 1, 12, 1).expect("invalid date"),
                    chrono::Weekday::Thu,
                ),
                chrono::Weekday::Sun,
            ),
            advent_next: Year::previous_inclusive(
                Year::next_inclusive(
                    NaiveDate::from_ymd_opt(year, 12, 1).expect("invalid date"),
                    chrono::Weekday::Thu,
                ),
                chrono::Weekday::Sun,
            ),
            ash_wednesday: easter - Duration::days(46),
            palm_sunday: easter - Duration::days(7),
            easter_sunday_2: easter + Duration::days(7), // 2nd Sunday of Easter (checked)
            pentecost: easter + Duration::days(49),
            annunciation: NaiveDate::from_ymd_opt(year, 3, 25).expect("invalid date"),
            christmas_next: NaiveDate::from_ymd_opt(year, 12, 25).expect("invalid date"),
            presentation: NaiveDate::from_ymd_opt(year, 2, 2).expect("invalid date"),
            all_saints: NaiveDate::from_ymd_opt(year, 11, 1).expect("invalid date"),
        }
    }

    /// find the date (of a [calendar::Holyday]) derived from a [calendar::DateCal] in the current
    /// year.
    fn date_cal_to_date(&self, date_cal: &calendar::DateCal) -> Result<NaiveDate> {
        Ok(match date_cal {
            calendar::DateCal::Easter => self.easter,
            calendar::DateCal::Advent => self.advent_previous,
            calendar::DateCal::AdventNext => self.advent_next,
            calendar::DateCal::Fixed { month, day } => {
                NaiveDate::from_ymd_opt(self.ad, u32::from(*month), u32::from(*day))
                    .ok_or_else(|| CalendarError::new(&format!("invalid date: {month}/{day}")))?
            },
            calendar::DateCal::After { date, rel } => {
                self.date_cal_to_date(date)? + Duration::days(i64::from(*rel))
            },
            calendar::DateCal::Next { date, day_of_week } =>
            // obsolete, do not use
            {
                Year::next_inclusive(
                    self.date_cal_to_date(date)?,
                    chrono::Weekday::from(day_of_week.clone()),
                )
            },
            calendar::DateCal::NextSunday { date } => {
                Year::next_inclusive(self.date_cal_to_date(date)?, chrono::Weekday::Sun)
            },
        })
    }

    /** Calculate the date of Easter Day. Returns result as number of days since March 0.

    Uses Michael Behrend's version of Clavius’s original method, see
    www.cantab.net/users/michael.behrend/algorithms/easter/pages/main.html */
    pub fn computus(year: i32) -> i32 {
        let c = year / 100;
        let d = (3 * c - 5) / 4;
        // (solar correction) + 10, also used for day of week
        let e = (8 * c + 13) / 25; // (lunar correction) + 5
        let f = year % 19; // (golden number of year) - 1
                           // Get q, where q = 53 - (Clavius epact), so that
                           //    q + 21 = date of Paschal full moon in days since March 0.
                           // Value on left of % is always >= 0, so no worry there.
        let mut q = (227 - 11 * f + d - e) % 30;
        if (q == 29) || ((q == 28) && (f >= 11)) {
            q -= 1;
        }
        // Get day of week of Paschal full moon (0 = Sun, 1 = Mon, ..., 6 = Sat)
        let w = (year + (year / 4) - d + q) % 7;
        // Get next Sunday strictly after Paschal full moon
        q + 28 - w
    }

    /** the next day being the specified weekday, not including the original date.

    ```
    use calendar::year_calendar::Year;
    use chrono::NaiveDate;
    use crate::calendar;

    let base = NaiveDate::from_ymd(2019, 6, 15);
    for (wd, date) in vec![
        (chrono::Weekday::Sun, 16),
        (chrono::Weekday::Sat, 22),
        (chrono::Weekday::Mon, 17),
    ] {
        let act = Year::next_exclusive(base, wd);
        let exp = NaiveDate::from_ymd(2019, 6, date);
        assert_eq!(exp, act, "exp {:?} act {:?}", exp, act);
    }
    ```
                         */
    pub fn next_exclusive(orig_date: NaiveDate, weekday: chrono::Weekday) -> NaiveDate {
        let orig_dow = orig_date.weekday().num_days_from_sunday() as i8; /* Sun = 0 etc */
        let req_dow = weekday.num_days_from_sunday() as i8; /* Sun = 0 etc */
        let offset = i64::from(req_dow - orig_dow + if req_dow <= orig_dow { 7 } else { 0 });
        orig_date + Duration::days(offset)
    }

    /** the next day being the specified weekday, including the original date.

    ```
    use chrono::NaiveDate;
    let base = NaiveDate::from_ymd(2019, 6, 15);
    for (wd, date) in vec![
        (chrono::Weekday::Sun, 16),
        (chrono::Weekday::Sat, 15),
        (chrono::Weekday::Mon, 17),
    ] {
        let act = calendar::year_calendar::Year::next_inclusive(base, wd);
        let exp = NaiveDate::from_ymd(2019, 6, date);
        assert_eq!(exp, act, "exp {:?} act {:?}", exp, act);
    }
    ```
                         */
    pub fn next_inclusive(orig_date: NaiveDate, weekday: chrono::Weekday) -> NaiveDate {
        let orig_dow = orig_date.weekday().num_days_from_sunday() as i8; /* Sun = 0 etc */
        let req_dow = weekday.num_days_from_sunday() as i8; /* Sun = 0 etc */
        let offset = i64::from(req_dow - orig_dow + if req_dow < orig_dow { 7 } else { 0 });
        orig_date + Duration::days(offset)
    }

    /** the most recent day being the specified weekday, excluding the original date.

    ```
    use chrono::NaiveDate;
    let base = NaiveDate::from_ymd(2019, 6, 15);
    for (wd, date) in vec![
        (chrono::Weekday::Sun, 9),
        (chrono::Weekday::Sat, 8),
        (chrono::Weekday::Mon, 10),
    ] {
        let act = calendar::year_calendar::Year::previous_exclusive(base, wd);
        let exp = NaiveDate::from_ymd(2019, 6, date);
        assert_eq!(exp, act, "exp {:?} act {:?}", exp, act);
    }
    ```
                         */
    pub fn previous_exclusive(orig_date: NaiveDate, weekday: chrono::Weekday) -> NaiveDate {
        let orig_dow = orig_date.weekday().num_days_from_sunday() as i8; /* Sun = 0 etc */
        let req_dow = weekday.num_days_from_sunday() as i8; /* Sun = 0 etc */
        let offset = i64::from(req_dow - orig_dow + if req_dow < orig_dow { 0 } else { -7 });
        orig_date + Duration::days(offset)
    }

    /** the most recent day being the specified weekday, including the original date.

    ```
    use chrono::NaiveDate;
    let base = NaiveDate::from_ymd(2019, 6, 15);
    for (wd, date) in vec![
        (chrono::Weekday::Sun, 9),
        (chrono::Weekday::Sat, 15),
        (chrono::Weekday::Mon, 10),
    ] {
        let act = calendar::year_calendar::Year::previous_inclusive(base, wd);
        let exp = NaiveDate::from_ymd(2019, 6, date);
        assert_eq!(exp, act, "exp {:?} act {:?}", exp, act);
    }
    ```
                         */
    pub fn previous_inclusive(orig_date: NaiveDate, weekday: chrono::Weekday) -> NaiveDate {
        let orig_dow = orig_date.weekday().num_days_from_sunday() as i8; /* Sun = 0 etc */
        let req_dow = weekday.num_days_from_sunday() as i8; /* Sun = 0 etc */
        let offset = i64::from(req_dow - orig_dow + if req_dow <= orig_dow { 0 } else { -7 });
        orig_date + Duration::days(offset)
    }

    /** The seasonal colour for a date.

    See [CoE Common Worship](https://www.churchofengland.org/prayer-and-worship/worship-texts-and-resources/common-worship/prayer-and-worship/worship-texts-and-resources/common-worship/churchs-year/rules).

    "White is the colour for the festal periods from Christmas
    Day to the Presentation and from Easter Day to the Eve of
    Pentecost, for Trinity Sunday, for Festivals of Our Lord and
    the Blessed Virgin Mary, for All Saints’ Day, and for the
    Festivals of those saints not venerated as martyrs, for the
    Feast of Dedication of a church, at Holy Communion on Maundy
    Thursday and in thanksgiving for Holy Communion and Holy
    Baptism..." */

    pub fn season_colour(
        &self,
        date: NaiveDate,
        advice: &mut Vec<String>,
    ) -> calendar::SeasonColour {
        if date >= self.christmas_next {
            advice.push("season colour is white because Christmas".to_string());
            calendar::SeasonColour::White
        } else if date <= self.presentation
            || (date >= self.easter && date < self.pentecost)
            || date == self.all_saints
        // TODO  for Trinity Sunday, for Festivals of Our Lord and the Blessed Virgin Mary,
        {
            advice.push("season colour is white because Epiphany/Easter/All Saints".to_string());
            calendar::SeasonColour::White
        }
        /* "Red is used during Holy Week (except at Holy Communion on
        Maundy Thursday), on the Feast of Pentecost... Coloured
        hangings are traditionally removed for Good Friday and
        Easter Eve, but red is the colour for the liturgy on Good
        Friday..." */
        else if (date >= self.palm_sunday && date < self.easter) || date == self.pentecost {
            advice.push("season colour is red because Easter/Pentecost".to_string());
            calendar::SeasonColour::Red
        }
        /* "Purple ... is the colour for Advent and from Ash Wednesday
        until the day before Palm Sunday..." */
        else if (date >= self.advent_next && date < self.christmas_next)
            || (date >= self.ash_wednesday && date < self.palm_sunday)
        {
            advice.push("season colour is purple because Advent/Lent".to_string());
            calendar::SeasonColour::Purple
        }
        /* "Green is used from the day after the Presentation until
        Shrove Tuesday, and from the day after Pentecost until the
        eve of All Saints’ Day, except when other provision is
        made..." */
        else {
            advice.push("season colour is green because Ordinary Time".to_string());
            calendar::SeasonColour::Green
        }
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
