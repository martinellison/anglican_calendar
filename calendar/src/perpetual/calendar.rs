/*! [Calendar] code */
use super::{FileInfo, HolydayRef, Province, Result};
use crate::perpetual::{CalendarError, Holyday};
/** A [Calendar] contains the [Holyday]s for a 'province' e.g. the Anglican
Church of Hong Kong. A Calendar is not specific to a specific year.*/
use ansi_term::Colour::*;
use askama::Template;
// use strum::IntoEnumIterator;
// use strum::{ EnumMessage};
use databake::{Bake, CrateEnv};
// use chrono::{Local, Utc, Weekday};
// use delegate::delegate;
// use itertools::Itertools;
// use log::debug;
use ron::{de::from_reader, ser::to_string_pretty};
use serde_derive::{Deserialize, Serialize};
use std::{
    // borrow::{Borrow, BorrowMut},
    collections::HashMap,
    // fmt::Write,
    io,
};
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Bake)]
#[databake(path = calendar::calendar)]
pub struct Calendar {
    #[serde(default)]
    /** info about the file */
    pub info: FileInfo,
    /** the province owning this calendar */
    pub province: Province,
    pub holydays: Vec<HolydayRef>,
    #[serde(skip)]
    pub holydays_by_tag: HashMap<String, HolydayRef>,
}

impl Default for Calendar {
    fn default() -> Self {
        Self {
            info: FileInfo::default(),
            province: Province::Unknown,
            holydays: vec![],
            holydays_by_tag: HashMap::new(),
        }
    }
}
impl Calendar {
    /** create an empty calendar */
    pub fn new() -> Self { Self::default() }

    /** add an [Holyday] to a [Calendar] */
    pub fn add(&mut self, holyday: &Holyday) {
        let r = HolydayRef::new(holyday.clone());
        self.holydays.push(r.clone());
        self.holydays_by_tag.insert(holyday.tag.clone(), r.clone());
    }

    /** read a calendar from a reader */
    pub fn read<R>(reader: R) -> Result<Self>
    where
        R: io::Read,
    {
        let mut u: Self = from_reader(reader).map_err(CalendarError::from_error)?;
        println!(
            "{}",
            Green.paint(format!("reading calendar for {:?}", u.province))
        );
        for r in &u.holydays {
            u.holydays_by_tag.insert(r.tag(), r.clone());
        }
        Ok(u)
    }

    /** write a [Calendar] to a writer. Prettyprint as it will probably be
     * saved. Is used */
    pub fn write<W>(&mut self, writer: &mut W) -> Result<()>
    where
        W: io::Write,
    {
        //self.holydays_by_tag.clear();
        let s = to_string_pretty(&self, ron::ser::PrettyConfig::default())
            .map_err(CalendarError::from_error)?;
        let _ = writer
            .write(s.as_bytes())
            .map_err(CalendarError::from_error)?;
        writer.flush().map_err(CalendarError::from_error)
    }

    // /** apply [EdMods] to the calendar */
    // pub fn apply(&mut self, edits: &EdMods) -> Result<()> {
    //     for em in &edits.holydays {
    //         match self.get_by_tag(&em.tag) {
    //             Ok(mut holyday) => {
    //                 if em.delete {
    //                     self.delete_by_tag(&em.tag);
    //                 } else {
    //                     holyday.modify(em);
    //                 }
    //             },
    //             Err(_e) => {
    //                 println!("tag {} not found, adding new holy day", &em.tag);
    //                 self.add(&em.to_holyday()?);
    //             },
    //         }
    //     }
    //     Ok(())
    // }

    // /** find the [Holyday] with a specified tag, or `None` */
    // pub fn get_by_tag(&mut self, tag: &str) -> Result<HolydayRef> {
    //     let re = self.holydays_by_tag.get(tag);
    //     if let Some(r) = re {
    //         Ok(r.clone())
    //     } else {
    //         Err(CalendarError::new(&format!("unknown tag {}", tag)))
    //     }
    // }

    // /** Remove an [Holyday] from the [Calendar].

    // The implementation is inefficient, but it should not be used very often. */
    // pub fn delete_by_tag(&mut self, tag: &str) {
    //     if let Some(index) = self.holydays.iter().position(|e| e.tag() == *tag) {
    //         self.holydays.swap_remove(index);
    //     }
    //     // if let  self.holydays.iter().position(|e| e.borrow().tag == *tag) {
    //     //     Some(index) => {
    //     //         self.holydays.swap_remove(index);
    //     //     }
    //     //     None => {}
    //     // }
    //     let _vo = self.holydays_by_tag.remove(tag);
    // }

    /** get all holy days for this [Calendar], in order (Principal holy days
     * first,...). */
    pub fn get_holydays(&self) -> Vec<HolydayRef> {
        let mut ee = self.holydays.clone();
        ee.sort();
        ee
    }

    /** `sort` sorts the calendar into a consistent order (using Ord). Is
     * used */
    pub fn sort(&mut self) { self.holydays.sort(); }

    /** `sort` sorts the calendar into a consistent order by date
     * calculation. */
    pub fn sort_by_date_cal(&mut self) { self.holydays.sort_by(HolydayRef::cmp_by_date_cal); }

    /** `sort` sorts the calendar into a consistent order by tag. */
    pub fn sort_by_tag(&mut self) { self.holydays.sort_by(HolydayRef::cmp_by_tag); }

    /** `clean_up` cleans up the data for the calendar */
    pub fn clean_up(&mut self) {
        for holy_day in &mut self.holydays {
            holy_day.clean_up();
        }
    }

    /** `dump_as_rust` dumps out the calendar as rust code */
    pub fn dump_as_rust(&self) -> String { self.bake(&CrateEnv::default()).to_string() }

    /** Write a human-readable report about the perpetual calendar to a file. */
    pub fn write_perpetual_report(&self, w: &mut dyn io::Write) -> Result<()> {
        let rt = PerpetualReportTemplate {
            province: self.province.to_string(),
            holydays: self.holydays.clone(),
        };
        // todo!("code perpetual report");
        let r = rt.render().map_err(CalendarError::from_error)?;
        w.write_all(r.as_bytes()).map_err(CalendarError::from_error)
    }
}

/// the template for the perpetual calendar report (for merging with the data)
#[derive(Template)]
#[template(path = "perpetual_report.html")]
struct PerpetualReportTemplate {
    province: String,
    holydays: Vec<HolydayRef>,
}
