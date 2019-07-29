/*! Implements a calendar for a specific year, as derived from a [calendar::Calendar] */
extern crate askama;
use crate::calendar;
use ansi_term::Color::Yellow;
use ansi_term::Colour::*;
use askama::Template;
use chrono::offset::TimeZone;
use chrono::Datelike;
use chrono::Duration;
use chrono::NaiveDate;
use chrono::Utc;
use icalendar::*;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::Write;

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
    pub fn from_calendar(
        calendar: &calendar::Calendar,
        year: i32,
        verbose: bool,
    ) -> Result<Self, calendar::CalendarError> {
        let y = Year::new(year);
        let mut ycal = Self {
            year: y.clone(),
            province: calendar.province,
            holydays_by_date: HashMap::new(),
        };
        for e in calendar.get_holydays() {
            let eb = e.borrow();
            let mut ye = YearHolyday::from_holyday(&e, &ycal.year);
            println!(
                "{}",
                Green.bold().paint(format!(
                    "converting {} ({:?}) {}",
                    eb.title,
                    eb.class,
                    ye.colour(&y)
                ))
            );
            ycal.add(&mut ye, &y, verbose)?;
        }
        Ok(ycal)
    }
    /** Generate an iCalendar. Also generate the calendar updates to cancel the entries.

    See [RFC 5545](https://tools.ietf.org/html/rfc5545) and [RFC
    7986](https://tools.ietf.org/html/rfc7986) for details of the
    iCalendar format. */

    pub fn to_ical(&self, unique: &str) -> (icalendar::Calendar, icalendar::Calendar) {
        let mut ical = icalendar::Calendar::new();

        /* TODO: add calendar properties here e.g. set
         * REFRESH-INTERVAL to P4W i.e. refresh every 4 weeks */

        let mut ical_del = icalendar::Calendar::new();
        println!("unique code for holydays is {}", unique);
        let mut ix = 0;
        let mut dates: Vec<&NaiveDate> = self.holydays_by_date.keys().collect();
        dates.sort();
        for d in dates {
            let yee = &self.holydays_by_date[&d];
            for ye in yee {
                let yeb = ye.holyday.borrow();
                let uid = format!("{}-{}", unique, ix);
                let mut e1a = icalendar::Event::new();
                let e = e1a
                    .summary(&yeb.title)
                    .description(&yeb.description)
                    .all_day(Utc.from_utc_date(&ye.date))
                    .uid(&uid)
                    .append_property(icalendar::Property::new("TRANSP", "TRANSPARENT"))
                    .append_property(icalendar::Property::new("SEQUENCE", "0"))
                    .append_property(icalendar::Property::new("COLOR", &ye.colour(&self.year)))
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
    pub fn write_report(&self, w: &mut dyn Write) -> Result<(), calendar::CalendarError> {
        let mut rt = ReportTemplate {
            dates: vec![],
            year: self.year.ad,
            province: self.province.to_string(),
        };
        let mut dates: Vec<&NaiveDate> = self.holydays_by_date.keys().collect();
        dates.sort();
        for d in dates {
            let season_colour = self.year.season_colour(*d);
            let mut rd = ReportDate {
                date: *d,
                date_form: d.format("%A %B %e").to_string(),
                holydays: vec![],
                colour_a: season_colour.colour_a(),
                colour_b: season_colour.colour_b(),
            };
            let yee = &self.holydays_by_date[&d];
            for ye in yee {
                let yeb = ye.holyday.borrow().clone();
                let mut refs_format: Vec<(String, String)> = vec![];
                for r in &yeb.refs {
                    refs_format.push((r.url(), r.description.clone()));
                }
                let rhd = ReportHolyday {
                    title: yeb.title.clone(),
                    description: yeb.description.clone(),
                    other: yeb.other.clone(),
                    refs_format,
                    class_format: yeb.class.to_string(),
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
    fn add(
        &mut self,
        ye: &mut YearHolyday,
        year: &Year,
        verbose: bool,
    ) -> Result<(), calendar::CalendarError> {
        if verbose {
            println!("for {} adding {}", ye.date, ye.holyday.borrow().title);
        }
        if let Some(day_holydays) = self.holydays_by_date.get_mut(&ye.date) {
            Self::add_holyday_if_ok(day_holydays, ye, year, verbose)?;
        } else {
            if verbose {
                println!("new date {}", ye.date);
            }
            let mut de = vec![];
            Self::add_holyday_if_ok(&mut de, ye, year, verbose)?;
            self.holydays_by_date.insert(ye.date, de); // may insert empty list, is ok
        };
        Ok(())
    }
    fn add_holyday_if_ok(
        day_holydays: &mut Vec<YearHolyday>,
        ye: &mut YearHolyday,
        year: &Year,
        _verbose: bool,
    ) -> Result<(), calendar::CalendarError> {
        let ds = Self::fix_holyday_date_is_ok(day_holydays, ye, year);
        match ds {
            DropStatus::Keep => {
                day_holydays.push(ye.clone());
            }
            DropStatus::Drop(r) => {
                // if verbose {
                println!(
                    "{}",
                    Yellow.bold().paint(format!(
                        "{} ({}) dropped because {:?}",
                        ye.holyday.borrow().title,
                        ye.date,
                        r
                    ))
                );
                // }
            }
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
    they appear on a Sumday but commemorations appearing on the same day
    will not be dropped.
                 */
    pub fn fix_holyday_date_is_ok(
        day_holydays: &[YearHolyday],
        ye: &mut YearHolyday,
        year: &Year,
    ) -> DropStatus {
        // calculate some dates and date ranges

        let is_sunday = ye.date.weekday() == chrono::Weekday::Sun;
        if is_sunday {
            println!("{} ({}) is sunday", ye.holyday.borrow().title, ye.date);
        }
        // let is_weekday = match ye.date.weekday() {
        //     chrono::Weekday::Sat | chrono::Weekday::Sun => false,
        //     _ => true,
        // };
        let is_in_advent =
            ye.date >= year.advent_next && ye.date < NaiveDate::from_ymd(year.ad, 12, 25);
        if is_in_advent {
            println!("{} ({}) is in advent", ye.holyday.borrow().title, ye.date);
        }
        let is_in_lent_or_eastertide = ye.date >= year.ash_wednesday && ye.date <= year.pentecost;
        if is_in_lent_or_eastertide {
            println!(
                "{} ({}) is in lent or eastertide",
                ye.holyday.borrow().title,
                ye.date
            );
        }
        //        let is_in_holy_week = ye.date >= year.palm_sunday && ye.date < year.easter;
        let is_in_easter = ye.date >= year.palm_sunday && ye.date <= year.easter_sunday_2;
        if is_in_easter {
            println!("{} ({}) is in easter", ye.holyday.borrow().title, ye.date);
        }

        let has_clash = !day_holydays.is_empty(); // TODO only if different levels
        let mut clash_level = calendar::HolydayClass::NotAFestival;
        if has_clash {
            for ce in day_holydays.iter() {
                let cel = ce.holyday.borrow().class;
                if cel > clash_level {
                    clash_level = cel
                }
            }
            print!(
                "date clash {} already {} holydays: ",
                &ye.date,
                day_holydays.len()
            );
            for e in day_holydays.iter() {
                print!("{}, ", e.holyday.borrow().title);
            }
            print!("{}", ye.holyday.borrow().title);
            println!();
        }

        let t = ye.holyday.borrow().transfer.clone();
        let c = ye.holyday.borrow().class;
        let clash_higher = has_clash && clash_level > c;
        match t {
            // TODO no 'saints days' in Easter Week
            calendar::TransferType::Normal => match c {
                calendar::HolydayClass::Commemoration => {
                    /* no transfer required */
                    if is_in_easter {
                        DropStatus::Drop(DropReason::Easter)
                    } else {
                        DropStatus::Keep
                    }
                }
                calendar::HolydayClass::LesserFestival => {
                    if is_in_easter {
                        DropStatus::Drop(DropReason::Easter)
                    } else if clash_higher {
                        DropStatus::Drop(DropReason::Clash)
                    } else if is_sunday {
                        DropStatus::Drop(DropReason::Sunday)
                    } else {
                        DropStatus::Keep
                    }
                }
                calendar::HolydayClass::Festival | calendar::HolydayClass::CorpusChristi => {
                    if (is_sunday && (is_in_advent || is_in_lent_or_eastertide)) || clash_higher {
                        ye.change_date_by(Duration::days(1))
                    }
                    DropStatus::Keep
                }
                calendar::HolydayClass::Principal => {
                    // if has_clash {
                    //     // Err(calendar::CalendarError::new(&format!(
                    //     //     "principal holyday {} may not be moved; reorder to start of holydays.",
                    //     //     ye.holyday.borrow().title
                    //     // )))
                    //     DropStatus::Drop(DropReason::Clash) // ??
                    // } else {
                    DropStatus::Keep
                    // }
                }
                calendar::HolydayClass::Unclassified => {
                    /* no transfer required? */
                    DropStatus::Keep
                }
                calendar::HolydayClass::NotAFestival => panic!("bad class"),
            },
            calendar::TransferType::Annunciation => {
                if is_sunday {
                    ye.change_date_by(Duration::days(1))
                }
                DropStatus::Keep
            }
            // calendar::TransferType::BaptismOfChrist => {
            //     Ok(true)
            // }
            calendar::TransferType::Joseph => {
                if is_in_easter {
                    let abvm = NaiveDate::from_ymd(year.ad, 3, 25);
                    let days = if abvm >= year.palm_sunday && abvm <= year.easter_sunday_2 {
                        2
                    } else {
                        1
                    };
                    ye.change_date_to(year.easter_sunday_2 + Duration::days(days));
                }
                DropStatus::Keep
            }
            calendar::TransferType::George => {
                if is_in_easter {
                    ye.change_date_to(year.easter_sunday_2 + Duration::days(1))
                }
                DropStatus::Keep
            }
            calendar::TransferType::Mark => {
                if is_in_easter {
                    let sgd = ye.date - Duration::days(2);
                    let days = if sgd >= year.palm_sunday && sgd <= year.easter_sunday_2 {
                        2
                    } else {
                        1
                    };
                    ye.change_date_to(year.easter_sunday_2 + Duration::days(days));
                }
                DropStatus::Keep
            }
            calendar::TransferType::Before(cutoff) => {
                if ye.date >= year.date_cal_to_date(&cutoff) {
                    DropStatus::Drop(DropReason::Cutoff)
                } else {
                    DropStatus::Keep
                }
            }
        }
    }
}
/** whether a [YearHolyday] will be dropped. */
#[derive(Debug, Eq, PartialEq)]
pub enum DropStatus {
    Keep,
    Drop(DropReason),
}
/** reason for dropping a [YearHolyday] */
#[derive(Debug, Eq, PartialEq)]
pub enum DropReason {
    Easter,
    Clash,
    Sunday,
    Cutoff,
    Other,
}
#[derive(Debug, Eq, PartialEq, Clone)]
/** A YearHolyday is an holyday in the calendar for a specific year
([YearCalendar]) e.g. in the 2019 calendar of the Anglican Church of
Hong Kong, Easter Sunday was 21 April and Matteo Ricci was 11 May. */
pub struct YearHolyday {
    holyday: calendar::HolydayRef,
    date: NaiveDate,
}
impl YearHolyday {
    /** Create a [YearHolyday] from an [calendar::Holyday] given the [Year] data. */
    pub fn from_holyday(holyday: &calendar::HolydayRef, year: &Year) -> Self {
        Self {
            holyday: holyday.clone(),
            date: year.date_cal_to_date(&holyday.borrow().date_cal),
        }
    }
    /** Change the date of a [YearHolyday] by a specified [Duration] */
    pub fn change_date_by(&mut self, cd: Duration) {
        self.date += cd;
        println!(
            "{}",
            Yellow.bold().paint(format!(
                "{} ({:?}/{:?}) changed to {} (modified by {:?})",
                &self.holyday.borrow().title,
                &self.holyday.borrow().class,
                &self.holyday.borrow().transfer,
                &self.date,
                cd
            ))
        );
    }
    /** set the date of a [YearHolyday] to the specified date */
    pub fn change_date_to(&mut self, d: NaiveDate) {
        self.date = d;
        println!(
            "{}",
            Yellow.bold().paint(format!(
                "{} ({:?}/{:?}) changed to {}",
                &self.holyday.borrow().title,
                &self.holyday.borrow().class,
                &self.holyday.borrow().transfer,
                &self.date
            ))
        );
    }
    /** the display colour for this holy day, from the CSS3 set of
    colour names, see https://www.w3.org/TR/css-color-3 */
    pub fn colour(&self, year: &Year) -> String {
        let hd = self.holyday.borrow();
        /* "If the Collect, Readings, etc. on a Lesser Festival are
        those of the saint, then either red (for a martyr) or white is
        used; " */
        match hd.class {
            // calendar::HolydayClass::Principal
            // | calendar::HolydayClass::CorpusChristi
            // |
            calendar::HolydayClass::Festival | calendar::HolydayClass::LesserFestival => {
                let is_martyr = hd.main.contains(&calendar::MainAttribute::Martyr);
                let fest_col = if is_martyr { "red" } else { "white" };
                println!(
                    "{} has colour {} for {:?}",
                    self.holyday.borrow().title,
                    fest_col,
                    hd.class
                );
                fest_col.to_string()
            }
            _ => match year.season_colour(self.date) {
                calendar::SeasonColour::White => "white".to_string(),
                calendar::SeasonColour::Red => "red".to_string(),
                calendar::SeasonColour::Purple => "purple".to_string(),
                calendar::SeasonColour::Green => "green".to_string(),
            },
        }
    }
}
impl Ord for YearHolyday {
    fn cmp(&self, other: &Self) -> Ordering {
        self.date.cmp(&other.date)
    }
}

impl PartialOrd for YearHolyday {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
#[derive(Template)]
#[template(path = "report.html")]
struct ReportTemplate {
    province: String,
    year: i32,
    dates: Vec<ReportDate>,
}
#[derive(Debug, Clone)]
struct ReportDate {
    date: NaiveDate,
    date_form: String,
    holydays: Vec<ReportHolyday>,
    colour_a: String,
    colour_b: String,
}
#[derive(Debug, Clone)]
struct ReportHolyday {
    title: String,
    description: String,
    class_format: String,
    other: Vec<String>,
    refs_format: Vec<(String, String)>,
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
}
impl Year {
    /** a year with calculated dates
     ```
    use anglican_calendar::year_calendar::Year;
    use chrono::NaiveDate;
    assert_eq!(
        Year {
            ad: 2020,
            easter: NaiveDate::from_ymd(2020, 4, 12),
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
        let easter =
            NaiveDate::from_ymd(year, 3, 1) + Duration::days(i64::from(Self::computus(year) - 1));
        Self {
            ad: year,
            easter,
            advent_previous: Year::previous_inclusive(
                Year::next_inclusive(NaiveDate::from_ymd(year - 1, 12, 1), chrono::Weekday::Thu),
                chrono::Weekday::Sun,
            ),
            advent_next: Year::previous_inclusive(
                Year::next_inclusive(NaiveDate::from_ymd(year, 12, 1), chrono::Weekday::Thu),
                chrono::Weekday::Sun,
            ),
            ash_wednesday: easter - Duration::days(46),
            palm_sunday: easter - Duration::days(7),
            easter_sunday_2: easter + Duration::days(7), // 2nd Sunday of Easter (checked)
            pentecost: easter + Duration::days(49),
        }
    }
    fn date_cal_to_date(&self, date_cal: &calendar::DateCal) -> NaiveDate {
        match date_cal {
            calendar::DateCal::Easter => self.easter,
            calendar::DateCal::Advent => self.advent_previous,
            calendar::DateCal::AdventNext => self.advent_next,
            calendar::DateCal::Fixed { month, day } => {
                NaiveDate::from_ymd(self.ad, u32::from(*month), u32::from(*day))
            }
            calendar::DateCal::After { date, rel } => {
                self.date_cal_to_date(date) + Duration::days(i64::from(*rel))
            }
            calendar::DateCal::Next { date, day_of_week } => Year::next_inclusive(
                self.date_cal_to_date(date),
                chrono::Weekday::from(day_of_week.clone()),
            ),
        }
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
    use chrono::NaiveDate;
    let base = NaiveDate::from_ymd(2019, 6, 15);
    for (wd, d) in vec![
        (chrono::Weekday::Sun, 16),
        (chrono::Weekday::Sat, 22),
        (chrono::Weekday::Mon, 17),
    ] {
        let act = anglican_calendar::year_calendar::Year::next_exclusive(base, wd);
        let exp = NaiveDate::from_ymd(2019, 6, d);
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
    for (wd, d) in vec![
        (chrono::Weekday::Sun, 16),
        (chrono::Weekday::Sat, 15),
        (chrono::Weekday::Mon, 17),
    ] {
        let act = anglican_calendar::year_calendar::Year::next_inclusive(base, wd);
        let exp = NaiveDate::from_ymd(2019, 6, d);
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
    for (wd, d) in vec![
        (chrono::Weekday::Sun, 9),
        (chrono::Weekday::Sat, 8),
        (chrono::Weekday::Mon, 10),
    ] {
        let act = anglican_calendar::year_calendar::Year::previous_exclusive(base, wd);
        let exp = NaiveDate::from_ymd(2019, 6, d);
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
    for (wd, d) in vec![
        (chrono::Weekday::Sun, 9),
        (chrono::Weekday::Sat, 15),
        (chrono::Weekday::Mon, 10),
    ] {
        let act = anglican_calendar::year_calendar::Year::previous_inclusive(base, wd);
        let exp = NaiveDate::from_ymd(2019, 6, d);
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
    /** The seasonal colour for a date. See [https://www.churchofengland.org/prayer-and-worship/worship-texts-and-resources/common-worship/prayer-and-worship/worship-texts-and-resources/common-worship/churchs-year/rules].
     */
    pub fn season_colour(&self, date: NaiveDate) -> calendar::SeasonColour {
        /* "White is the colour for the festal periods from Christmas
        Day to the Presentation and from Easter Day to the Eve of
        Pentecost, for Trinity Sunday, for Festivals of Our Lord and
        the Blessed Virgin Mary, for All Saints’ Day, and for the
        Festivals of those saints not venerated as martyrs, for the
        Feast of Dedication of a church, at Holy Communion on Maundy
        Thursday and in thanksgiving for Holy Communion and Holy
        Baptism..." */

        if date >= NaiveDate::from_ymd(self.ad, 12, 25)
            || date <= NaiveDate::from_ymd(self.ad, 2, 2)
            || (date >= self.easter && date < self.pentecost)
            || date == NaiveDate::from_ymd(self.ad, 11, 1)
        // TODO  for Trinity Sunday, for Festivals of Our Lord and the Blessed Virgin Mary,
        {
            calendar::SeasonColour::White
        }
        /* "Red is used during Holy Week (except at Holy Communion on
         * Maundy Thursday), on the Feast of Pentecost... Coloured
         * hangings are traditionally removed for Good Friday and
         * Easter Eve, but red is the colour for the liturgy on Good
         * Friday..." */
        else if (date >= self.palm_sunday && date < self.easter) || date == self.pentecost {
            calendar::SeasonColour::Red
        }
        /* "Purple ... is the colour for Advent and from Ash Wednesday
        until the day before Palm Sunday..." */
        else if (date >= self.advent_next && date < NaiveDate::from_ymd(self.ad, 12, 25))
            || (date >= self.ash_wednesday && date < self.palm_sunday)
        {
            calendar::SeasonColour::Purple
        }
        /* "Green is used from the day after the Presentation until
        Shrove Tuesday, and from the day after Pentecost until the
        eve of All Saints’ Day, except when other provision is
        made..." */
        else {
            calendar::SeasonColour::Green
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::year_calendar::{Year, YearCalendar, YearHolyday};
    use calendar::{DateCal, Holyday, TransferType};
    use std::cell::RefCell;
    use std::rc::Rc;
    #[test]
    fn test_transfers() {
        let day_holydays: Vec<YearHolyday> = vec![];
        let year = Year::new(2019);
        let mut ye = YearHolyday::from_holyday(&Rc::new(RefCell::new(Holyday::default())), &year);
        let ye_exp = ye.clone();
        let er = YearCalendar::fix_holyday_date_is_ok(&day_holydays, &mut ye, &year);
        assert_eq!(DropStatus::Keep, er);
        assert_eq!(ye_exp, ye, "bad holyday {:?}", ye);
    }
    #[test]
    fn test_easter() {
        let day_holydays: Vec<YearHolyday> = vec![];
        let year = Year::new(2019);
        let holyday = Holyday {
            title: "EASTER DAY".to_string(),
            description: "EASTER DAY, the first Sunday after the Paschal full moon".to_string(),
            class: calendar::HolydayClass::Principal,
            tag: "easter day".to_string(),
            date_cal: calendar::DateCal::Easter,
            transfer: calendar::TransferType::Normal,
            ..Holyday::default()
        };
        let mut ye = YearHolyday::from_holyday(&Rc::new(RefCell::new(holyday)), &year);
        let er = YearCalendar::fix_holyday_date_is_ok(&day_holydays, &mut ye, &year);
        assert_eq!(DropStatus::Keep, er);
        assert_eq!(NaiveDate::from_ymd(2019, 4, 21), ye.date);
    }
    // TODO add more test dates to cover all special cases
    #[test]
    fn test_dates_2019() {
        let year = 2019;
        let tests: Vec<(DateCal, TransferType, NaiveDate)> = vec![
            (
                DateCal::After {
                    date: Box::new(DateCal::Easter),
                    rel: -46,
                },
                TransferType::Normal,
                NaiveDate::from_ymd(year, 3, 6),
            ),
            (
                DateCal::Easter,
                TransferType::Normal,
                NaiveDate::from_ymd(year, 4, 21),
            ),
            (
                DateCal::After {
                    date: Box::new(DateCal::Easter),
                    rel: 39,
                },
                TransferType::Normal,
                NaiveDate::from_ymd(year, 5, 30),
            ),
            (
                DateCal::After {
                    date: Box::new(DateCal::Easter),
                    rel: 49,
                },
                TransferType::Normal,
                NaiveDate::from_ymd(year, 6, 9),
            ),
            (
                DateCal::Fixed { month: 12, day: 25 },
                TransferType::Normal,
                NaiveDate::from_ymd(year, 12, 25),
            ),
            (
                DateCal::Fixed { month: 4, day: 23 },
                TransferType::George,
                NaiveDate::from_ymd(year, 4, 29), // check!!
            ),
            // TODO: Advent Sunday
            (
                DateCal::Fixed { month: 4, day: 25 },
                TransferType::Mark,
                NaiveDate::from_ymd(year, 4, 30), // check!!
            ),
        ];
        test_year(year, &tests);
    }
    #[test]
    fn test_dates_2020() {
        let year = 2020;
        let tests: Vec<(DateCal, TransferType, NaiveDate)> = vec![
            (
                DateCal::After {
                    date: Box::new(DateCal::Easter),
                    rel: -46,
                },
                TransferType::Normal,
                NaiveDate::from_ymd(year, 2, 26),
            ),
            (
                DateCal::Easter,
                TransferType::Normal,
                NaiveDate::from_ymd(year, 4, 12),
            ),
            (
                DateCal::After {
                    date: Box::new(DateCal::Easter),
                    rel: 39,
                },
                TransferType::Normal,
                NaiveDate::from_ymd(year, 5, 21),
            ),
            (
                DateCal::After {
                    date: Box::new(DateCal::Easter),
                    rel: 49,
                },
                TransferType::Normal,
                NaiveDate::from_ymd(year, 5, 31),
            ),
            (
                DateCal::Fixed { month: 12, day: 25 },
                TransferType::Normal,
                NaiveDate::from_ymd(year, 12, 25),
            ),
            (
                DateCal::Fixed { month: 4, day: 23 },
                TransferType::George,
                NaiveDate::from_ymd(year, 4, 23), // check!!
            ),
            (
                DateCal::Fixed { month: 4, day: 25 },
                TransferType::Mark,
                NaiveDate::from_ymd(year, 4, 25), // check!!
            ),
            // TODO: Advent Sunday
        ];
        test_year(year, &tests);
    }
    fn test_year(year_ad: i32, tests: &Vec<(DateCal, TransferType, NaiveDate)>) {
        let day_holydays: Vec<YearHolyday> = vec![];
        let year = Year::new(year_ad);
        for (dc, t, ed) in tests {
            let holyday = Holyday {
                title: "test".to_string(),
                description: "test descr".to_string(),
                class: calendar::HolydayClass::Principal,
                tag: "test tag".to_string(),
                date_cal: dc.clone(),
                transfer: t.clone(),
                ..Holyday::default()
            };
            let mut ye = YearHolyday::from_holyday(&Rc::new(RefCell::new(holyday)), &year);
            let er = YearCalendar::fix_holyday_date_is_ok(&day_holydays, &mut ye, &year);
            assert_eq!(DropStatus::Keep, er);
            assert_eq!(
                *ed,
                ye.date,
                "wrong date, actual week day {:?}",
                ye.date.weekday()
            );
        }
    }
}
/*

Copyright ©2019 Martin Ellison.  This program is free software: you
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
<https://www.gnu.org/licenses/>. */
