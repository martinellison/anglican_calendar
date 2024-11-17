use super::*;
use crate::{
    perpetual::{DateCal, Holyday, TransferType},
    year_calendar::HolydayRef,
};
use chrono::Datelike;
use year_calendar::YearCalendar;

#[test]
fn test_nearby_sundays() {
    let year = Year::new(2024);
    // note: not used in HKSKH
    let d = year.nearby_sunday(NaiveDate::from_ymd_opt(2024, 1, 6).unwrap(), -4);
    assert_eq!(
        d,
        NaiveDate::from_ymd_opt(2024, 1, 7).unwrap(),
        "Epiphany 2024"
    );
    let d = year.nearby_sunday(NaiveDate::from_ymd_opt(2024, 2, 2).unwrap(), -5);
    assert_eq!(
        d,
        NaiveDate::from_ymd_opt(2024, 2, 4).unwrap(),
        "Candlemas 2024"
    );
    let d = year.nearby_sunday(NaiveDate::from_ymd_opt(2024, 11, 1).unwrap(), -2);
    assert_eq!(
        d,
        NaiveDate::from_ymd_opt(2024, 11, 3).unwrap(),
        "All Saints' 2024"
    );
    // FUTURE add more cases
}
#[test]
fn test_transfers() {
    let day_holydays: Vec<YearHolyday> = vec![];
    let year = Year::new(2019);
    let mut year_holyday =
        YearHolyday::from_holyday(&HolydayRef::new(Holyday::default()), &year, false).unwrap();
    let ye_exp = year_holyday.clone();
    let er = YearCalendar::fix_holyday_date_is_ok(&day_holydays, &mut year_holyday, &year);
    assert_eq!(DropStatus::Keep, er);
    assert_eq!(ye_exp, year_holyday, "bad holyday {:?}", year_holyday);
}
#[test]
fn test_easter() {
    let day_holydays: Vec<YearHolyday> = vec![];
    let year = Year::new(2019);
    let holyday = Holyday {
        title: "EASTER DAY".to_string(),
        description: "EASTER DAY, the first Sunday after the Paschal full moon".to_string(),
        class: HolydayClass::Principal,
        tag: "easter day".to_string(),
        date_cal: DateCal::Easter,
        transfer: TransferType::Normal,
        ..Holyday::default()
    };
    let mut year_holyday =
        YearHolyday::from_holyday(&HolydayRef::new(holyday), &year, false).unwrap();
    let er = YearCalendar::fix_holyday_date_is_ok(&day_holydays, &mut year_holyday, &year);
    assert_eq!(DropStatus::Keep, er);
    assert_eq!(
        NaiveDate::from_ymd_opt(2019, 4, 21),
        Some(year_holyday.date)
    );
}
// TODO add more test dates to cover all special cases
#[test]
fn test_dates_2019() {
    let year = 2019;
    let tests: Vec<(DateCal, TransferType, Option<NaiveDate>)> = vec![
        (
            DateCal::After {
                date: Box::new(DateCal::Easter),
                rel: -46,
            },
            TransferType::Normal,
            NaiveDate::from_ymd_opt(year, 3, 6),
        ),
        (
            DateCal::Easter,
            TransferType::Normal,
            NaiveDate::from_ymd_opt(year, 4, 21),
        ),
        (
            DateCal::After {
                date: Box::new(DateCal::Easter),
                rel: 39,
            },
            TransferType::Normal,
            NaiveDate::from_ymd_opt(year, 5, 30),
        ),
        (
            DateCal::After {
                date: Box::new(DateCal::Easter),
                rel: 49,
            },
            TransferType::Normal,
            NaiveDate::from_ymd_opt(year, 6, 9),
        ),
        (
            DateCal::Fixed { month: 12, day: 25 },
            TransferType::Normal,
            NaiveDate::from_ymd_opt(year, 12, 25),
        ),
        (
            DateCal::Fixed { month: 4, day: 23 },
            TransferType::George,
            NaiveDate::from_ymd_opt(year, 4, 29), // check!!
        ),
        // TODO: Advent Sunday
        (
            DateCal::Fixed { month: 4, day: 25 },
            TransferType::Mark,
            NaiveDate::from_ymd_opt(year, 4, 30), // check!!
        ),
    ];
    test_year(year, &tests);
}
#[test]
fn test_dates_2020() {
    let year = 2020;
    let tests: Vec<(DateCal, TransferType, Option<NaiveDate>)> = vec![
        (
            DateCal::After {
                date: Box::new(DateCal::Easter),
                rel: -46,
            },
            TransferType::Normal,
            NaiveDate::from_ymd_opt(year, 2, 26),
        ),
        (
            DateCal::Easter,
            TransferType::Normal,
            NaiveDate::from_ymd_opt(year, 4, 12),
        ),
        (
            DateCal::After {
                date: Box::new(DateCal::Easter),
                rel: 39,
            },
            TransferType::Normal,
            NaiveDate::from_ymd_opt(year, 5, 21),
        ),
        (
            DateCal::After {
                date: Box::new(DateCal::Easter),
                rel: 49,
            },
            TransferType::Normal,
            NaiveDate::from_ymd_opt(year, 5, 31),
        ),
        (
            DateCal::Fixed { month: 12, day: 25 },
            TransferType::Normal,
            NaiveDate::from_ymd_opt(year, 12, 25),
        ),
        (
            DateCal::Fixed { month: 4, day: 23 },
            TransferType::George,
            NaiveDate::from_ymd_opt(year, 4, 23), // check!!
        ),
        (
            DateCal::Fixed { month: 4, day: 25 },
            TransferType::Mark,
            NaiveDate::from_ymd_opt(year, 4, 25), // check!!
        ),
        // TODO: Advent Sunday
    ];
    test_year(year, &tests);
}

#[test]
fn test_dates_2024() {
    let year = 2024;
    let tests: Vec<(DateCal, TransferType, Option<NaiveDate>)> = vec![
        (
            DateCal::NextSunday {
                date: Box::new(DateCal::Fixed { month: 1, day: 6 }),
            },
            TransferType::Normal,
            NaiveDate::from_ymd_opt(year, 1, 7),
        ),
        (
            DateCal::After {
                date: Box::new(DateCal::Easter),
                rel: -46,
            },
            TransferType::Normal,
            NaiveDate::from_ymd_opt(year, 2, 14),
        ),
        (
            DateCal::After {
                date: Box::new(DateCal::Easter),
                rel: 56,
            },
            TransferType::Normal,
            NaiveDate::from_ymd_opt(year, 5, 26),
        ),
        (
            DateCal::After {
                date: Box::new(DateCal::Easter),
                rel: 60,
            },
            TransferType::Normal,
            NaiveDate::from_ymd_opt(year, 5, 30),
        ),
    ];
    test_year(year, &tests);
}

fn test_year(year_ad: i32, tests: &Vec<(DateCal, TransferType, Option<NaiveDate>)>) {
    let day_holydays: Vec<YearHolyday> = vec![];
    let year = Year::new(year_ad);
    for (dc, t, ed_opt) in tests {
        let holyday = Holyday {
            title: "test".to_string(),
            description: "test descr".to_string(),
            class: HolydayClass::Principal,
            tag: "test tag".to_string(),
            date_cal: dc.clone(),
            transfer: *t,
            ..Holyday::default()
        };
        let mut year_holyday =
            YearHolyday::from_holyday(&HolydayRef::new(holyday), &year, false).unwrap();
        let er = YearCalendar::fix_holyday_date_is_ok(&day_holydays, &mut year_holyday, &year);
        assert_eq!(DropStatus::Keep, er);
        assert_eq!(
            *ed_opt,
            Some(year_holyday.date),
            "wrong date, actual week day {:?}",
            year_holyday.date().weekday()
        );
    }
}

// TODO test more than one holy day together to check transfer logic

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
