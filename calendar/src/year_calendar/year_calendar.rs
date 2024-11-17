/*! [YearCalendar] code */
use crate::{
    perpetual::{calendar::Calendar, CalendarError, Province, TransferType},
    year_calendar::{
        DropReason, DropStatus, HolydayClass, ReportDate, ReportHolyday, ReportTemplate, Result,
        WallTemplate, Year, YearHolyday,
    },
};
use ansi_term::Colour::*;
use askama::Template;
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use icalendar::{Component, EventLike};
use log::debug;
// use icalendar::*;
use std::{collections::HashMap, io::Write};

#[derive(Debug, Eq, PartialEq, Clone)]
/** A YearCalendar is a calendar for a specific year for a specific
church e.g. the 2019 calendar of the Anglican Church of Hong Kong. */
pub struct YearCalendar {
    province: Province,
    pub(crate) year: Year,
    pub(crate) holydays_by_date: HashMap<NaiveDate, Vec<YearHolyday>>,
}
impl YearCalendar {
    /** Create a YearCalendar from a [Calendar] given the year. */
    pub fn from_calendar(calendar: &Calendar, year: i32, verbose: bool) -> Result<Self> {
        let y = Year::new(year);
        let mut ycal = Self {
            year: y.clone(),
            province: calendar.province,
            holydays_by_date: HashMap::new(),
        };
        for hd in calendar.get_holydays() {
            let mut year_holyday = YearHolyday::from_holyday(&hd, &ycal.year, false)?;
            ycal.add(&mut year_holyday, &y, verbose, true)?; // TODO program option keep_dropped
            if hd.has_eve() {
                let mut year_holyday_eve = YearHolyday::from_holyday(&hd, &ycal.year, true)?;
                ycal.add(&mut year_holyday_eve, &y, verbose, true)?; // TODO program option keep_dropped
            }
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
                    ));
                let refs = year_holyday.holyday.refs();
                if !refs.is_empty() {
                    let r#ref = &refs[0];
                    let desc = r#ref.description.trim();
                    if let Ok(url) = r#ref.url() {
                        let url = url.to_string();
                        if !desc.is_empty() {
                            debug!(
                                "adding url \"{}\" to \"{}\"",
                                &url,
                                &year_holyday.holyday.title()
                            );
                            // somehow it inserts a backslash into the property
                            e.append_property(icalendar::Property::new("URL", &url));
                        }
                    }
                }
                ical.push(e.done());
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
        let rt = ReportTemplate {
            dates: self.report_dates(),
            year: self.year.ad,
            province: self.province.to_string(),
        };
        let r = rt.render().map_err(CalendarError::from_error)?;
        w.write_all(r.as_bytes()).map_err(CalendarError::from_error)
    }

    /** `write_wall_calendar` writes a report in wall calendar format */
    pub fn write_wall_calendar(&self, w: &mut dyn Write) -> Result<()> {
        let rt = WallTemplate::new(
            self.province.to_string(),
            self.year.ad,
            &self.report_dates(),
        );
        let r = rt.render().map_err(CalendarError::from_error)?;
        w.write_all(r.as_bytes()).map_err(CalendarError::from_error)
    }

    /** `report_dates` provides the [ReportDate]s for the [Year] */
    pub fn report_dates(&self) -> Vec<ReportDate> {
        let mut dates: Vec<&NaiveDate> = self.holydays_by_date.keys().collect();
        dates.sort();
        let mut report_dates = vec![];
        for date in dates {
            let mut advice = vec![];
            // let season_colour = self.year.season_colour(*date, &mut advice);
            let mut rd = ReportDate {
                date: *date,
                date_form: date.format("%A %B %e").to_string(),
                holydays: vec![],
            };
            let year_holydays = &self.holydays_by_date[date];
            for year_holyday in year_holydays {
                let colour = year_holyday.colour(&self.year, &mut advice);
                let mut refs_format: Vec<(String, String)> = vec![];
                for r in &year_holyday.holyday.refs() {
                    let desc = r.description.trim();
                    if !desc.is_empty() {
                        refs_format.push((r.url().unwrap().to_string(), desc.to_string()));
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
                    is_eve: year_holyday.is_eve,
                };
                rd.holydays.push(rhd);
            }
            report_dates.push(rd);
        }
        report_dates
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
            self.holydays_by_date.insert(year_holyday.date, de);
            // may insert empty list, is ok
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
                "{} ({}) is a Sunday",
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
                "{} ({}) is in Advent",
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
        let mut clash_level = HolydayClass::NotAFestival;
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
            TransferType::Normal => {
                match c {
                    HolydayClass::Commemoration => {
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
                    HolydayClass::LesserFestival => {
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
                    HolydayClass::Festival | HolydayClass::CorpusChristi => {
                        if (is_sunday && (is_in_advent || is_in_lent_or_eastertide)) || clash_higher
                        {
                            year_holyday.add_advice("Festival or Corpus Christi date changed");
                            year_holyday.change_date_by(Duration::days(1))
                        }
                        DropStatus::Keep
                    },
                    HolydayClass::Principal => {
                        // if day_has_holyday {
                        //     // Err(CalendarError::new(&format!(
                        //     //     "principal holyday {} may not be moved; reorder to start of
                        // holydays.",     //     year_holyday.holyday.borrow().title
                        //     // )))
                        //     DropStatus::Drop(DropReason::Clash) // ??
                        // } else {
                        DropStatus::Keep
                        // }
                    },
                    HolydayClass::Sunday => {
                        /* what about Annunciation?? */
                        assert!(is_sunday);
                        if clash_higher {
                            year_holyday.add_advice("Sunday clash");
                            DropStatus::Drop(DropReason::Clash)
                        } else {
                            DropStatus::Keep
                        }
                    },
                    HolydayClass::Unclassified => {
                        /* no transfer required? */
                        DropStatus::Keep
                    },
                    HolydayClass::NotAFestival => panic!("bad class"),
                }
            },
            TransferType::Annunciation => {
                if is_in_easter {
                    year_holyday.add_advice("Annunciation transferred because in Easter");
                    year_holyday.change_date_to(year.easter_sunday_2 + Duration::days(1));
                } else if is_sunday {
                    year_holyday.add_advice("Annunciation transferred because on Sunday");
                    year_holyday.change_date_by(Duration::days(1))
                }
                DropStatus::Keep
            },
            // TransferType::BaptismOfChrist => {
            //     Ok(true)
            // }
            TransferType::Joseph => {
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
            TransferType::George => {
                if is_in_easter {
                    year_holyday.add_advice("date change");
                    year_holyday.change_date_to(year.easter_sunday_2 + Duration::days(1))
                }
                DropStatus::Keep
            },
            TransferType::Mark => {
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
            // TransferType::BeforeAdventNext => {
            //     if year_holyday.date >= year.advent_next {
            //         DropStatus::Drop(DropReason::Cutoff)
            //     } else {
            //         DropStatus::Keep
            //     }
            // },
            TransferType::DoNotTransfer => DropStatus::Keep,
            TransferType::EpiphanyOption => {
                // if the date is already a Sunday, it is not changed
                // for Epiphany (-4:+2)  at local option
                year_holyday.add_advice("Epiphany transferred to nearby Sunday");
                year_holyday.change_date_to(year.nearby_sunday(year_holyday.date, -4));
                DropStatus::Keep
            },
            TransferType::PresentationOption => {
                // if the date is already a Sunday, it is not changed
                // for Presentation(-5:+1) at local option
                year_holyday.add_advice("Presentation transferred to nearby Sunday");
                year_holyday.change_date_to(year.nearby_sunday(year_holyday.date, -5));
                DropStatus::Keep
            },
            TransferType::AllSaintsOption => {
                // if the date is already a Sunday, it is not changed
                // for All Saints'(-2:+4) at local option
                year_holyday.add_advice("All Saints' transferred to nearby Sunday");
                year_holyday.change_date_to(year.nearby_sunday(year_holyday.date, -2));
                DropStatus::Keep
            },
            TransferType::TransferNextSunday => {
                // if the date is already a Sunday, it is not changed
                year_holyday.add_advice("Transferred to the following Sunday");
                year_holyday.change_date_to(Year::next_inclusive(year_holyday.date, Weekday::Sun));
                DropStatus::Keep
            },
            TransferType::DropOnClash => {
                if clash_higher {
                    year_holyday.add_advice("Dropped because it coincides with another holy day");
                    DropStatus::Drop(DropReason::Clash)
                } else if is_sunday {
                    year_holyday.add_advice("Dropped because it occurs on Sunday");
                    DropStatus::Drop(DropReason::Sunday)
                } else {
                    DropStatus::Keep
                }
            },
            TransferType::NextDayOnClash => {
                if clash_higher {
                    // TODO check: do we ever have to move by 2 days or more?
                    year_holyday.change_date_by(Duration::days(1));
                    year_holyday.add_advice(
                        "Moved to the following day because it coincides with another holy day",
                    );
                }
                DropStatus::Keep
            },
            TransferType::KeepOnClash => {
                if clash_higher {
                    year_holyday.add_advice("Kept ever though it coincides with another holy day");
                }
                DropStatus::Keep
            },
        }
    }
}
