/*! code for [Year] */
use crate::perpetual::{CalendarError, DateCal, Result, SeasonColour};
use chrono::{Datelike, Duration, NaiveDate};
use icalendar::*;

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
    /** date of  Maundy Thursday */
    pub maundy_thursday: NaiveDate,
    /** date of the second Sunday in Easter */
    pub easter_sunday_2: NaiveDate,
    /** date of Pentecost */
    pub pentecost: NaiveDate,
    /** date of the Annunciation TODO what if moved? */
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
    use anglican_year_Year;
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
            maundy_thursday: easter - Duration::days(3),
            easter_sunday_2: easter + Duration::days(7), // 2nd Sunday of Easter (checked)
            pentecost: easter + Duration::days(49),
            annunciation: NaiveDate::from_ymd_opt(year, 3, 25).expect("invalid date"),
            christmas_next: NaiveDate::from_ymd_opt(year, 12, 25).expect("invalid date"),
            presentation: NaiveDate::from_ymd_opt(year, 2, 2).expect("invalid date"),
            all_saints: NaiveDate::from_ymd_opt(year, 11, 1).expect("invalid date"),
        }
    }

    /// find the date (of a [Holyday]) derived from a
    /// [DateCal] in the current year.
    pub(crate) fn date_cal_to_date(&self, date_cal: &DateCal) -> Result<NaiveDate> {
        Ok(match date_cal {
            DateCal::Easter => self.easter,
            DateCal::Advent => self.advent_previous,
            DateCal::AdventNext => self.advent_next,
            DateCal::Fixed { month, day } => {
                NaiveDate::from_ymd_opt(self.ad, u32::from(*month), u32::from(*day))
                    .ok_or_else(|| CalendarError::new(&format!("invalid date: {month}/{day}")))?
            },
            DateCal::After { date, rel } => {
                self.date_cal_to_date(date)? + Duration::days(i64::from(*rel))
            },
            DateCal::Next { date, day_of_week } =>
            // obsolete, do not use
            {
                Year::next_inclusive(
                    self.date_cal_to_date(date)?,
                    chrono::Weekday::from(day_of_week.clone()),
                )
            },
            DateCal::NextSunday { date } => {
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
    use chrono::NaiveDate;
    use crate::calendar;
    use calendar::year_calendar::Year;

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
    use calendar::year_calendar::Year;
    let base = NaiveDate::from_ymd(2019, 6, 15);
    for (wd, date) in vec![
        (chrono::Weekday::Sun, 16),
        (chrono::Weekday::Sat, 15),
        (chrono::Weekday::Mon, 17),
    ] {
        let act = Year::next_inclusive(base, wd);
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
    use calendar::year_calendar::Year;
    let base = NaiveDate::from_ymd(2019, 6, 15);
    for (wd, date) in vec![
        (chrono::Weekday::Sun, 9),
        (chrono::Weekday::Sat, 8),
        (chrono::Weekday::Mon, 10),
    ] {
        let act = Year::previous_exclusive(base, wd);
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
    use calendar::year_calendar::Year;
    let base = NaiveDate::from_ymd(2019, 6, 15);
    for (wd, date) in vec![
        (chrono::Weekday::Sun, 9),
        (chrono::Weekday::Sat, 15),
        (chrono::Weekday::Mon, 10),
    ] {
        let act = Year::previous_inclusive(base, wd);
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

    /** `nearby_sunday` finds a "nearby" Sunday */
    pub fn nearby_sunday(&self, orig_date: NaiveDate, max_back: i8) -> NaiveDate {
        let orig_dow = orig_date.weekday().num_days_from_sunday() as i8; /* Sun = 0 etc */
        let offset = i64::from(-orig_dow + if orig_dow < -max_back { 0 } else { 7 });

        // eprintln!("date {orig_date} is day {orig_dow}, so offset is {offset} and
        // nearby Sunday is {s}");
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

    pub fn season_colour(&self, date: NaiveDate, advice: &mut Vec<String>) -> SeasonColour {
        if date >= self.christmas_next {
            advice.push("season colour is white because Christmas".to_string());
            SeasonColour::White
        } else if date <= self.presentation
            || (date >= self.easter && date < self.pentecost)
            || date == self.all_saints
            || date == self.maundy_thursday
        // TODO  for Trinity Sunday, for Festivals of Our Lord and the Blessed Virgin Mary,
        {
            advice.push("season colour is white because Epiphany/Easter/All Saints".to_string());
            SeasonColour::White
        }
        /* "Red is used during Holy Week (except at Holy Communion on
        Maundy Thursday), on the Feast of Pentecost... Coloured
        hangings are traditionally removed for Good Friday and
        Easter Eve, but red is the colour for the liturgy on Good
        Friday..." */
        else if (date >= self.palm_sunday && date < self.easter) || date == self.pentecost {
            advice.push("season colour is red because Easter/Pentecost".to_string());
            SeasonColour::Red
        }
        /* "Purple ... is the colour for Advent and from Ash Wednesday
        until the day before Palm Sunday..." */
        else if (date >= self.advent_next && date < self.christmas_next)
            || (date >= self.ash_wednesday && date < self.palm_sunday)
        {
            advice.push("season colour is purple because Advent/Lent".to_string());
            SeasonColour::Purple
        }
        /* "Green is used from the day after the Presentation until
        Shrove Tuesday, and from the day after Pentecost until the
        eve of All Saints’ Day, except when other provision is
        made..." */
        else {
            advice.push("season colour is green because Ordinary Time".to_string());
            SeasonColour::Green
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
