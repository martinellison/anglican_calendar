/*! This module covers the year-independent data for a calendar.  */
//use serde::{Deserialize, Serialize};
//use chrono::prelude::*;
use ansi_term::Colour::*;
use ron::de::from_reader;
use ron::ser::to_string_pretty;
use serde_derive::{Deserialize, Serialize};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::io;
use std::rc::Rc;

/** A [Calendar] contains the [Holyday]s for a 'province' e.g. the Anglican
Church of Hong Kong. A Calendar is not specific to a specific year.*/
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Calendar {
    holydays: Vec<HolydayRef>,
    #[serde(skip)]
    holydays_by_tag: HashMap<String, HolydayRef>,
}
/** A reference-counted pointer to an [Holyday] */
pub type HolydayRef = Rc<RefCell<Holyday>>;
impl Calendar {
    /** create an empty calendar */
    pub fn new() -> Self {
        Self {
            holydays: vec![],
            holydays_by_tag: HashMap::new(),
        }
    }
    /** add an [Holyday] to a [Calendar] */
    pub fn add(&mut self, holyday: &Holyday) {
        let r = Rc::new(RefCell::new(holyday.clone()));
        self.holydays.push(r.clone());
        self.holydays_by_tag.insert(holyday.tag.clone(), r.clone());
    }
    /** read a calendar from a reader */
    pub fn read<R>(reader: R) -> Result<Self, CalendarError>
    where
        R: io::Read,
    {
        let mut u: Self = from_reader(reader).map_err(CalendarError::from_error)?;
        for r in &u.holydays {
            u.holydays_by_tag.insert(r.borrow().tag.clone(), r.clone());
        }
        Ok(u)
    }
    /** write a [Calendar] to a writer. Prettyprint as it will probably be saved. */
    pub fn write<W>(&mut self, writer: &mut W) -> Result<(), CalendarError>
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
    /** apply [EdMods] to the calendar */
    pub fn apply(&mut self, edits: &EdMods) -> Result<(), CalendarError> {
        for em in &edits.holydays {
            match self.get_by_tag(&em.tag) {
                Ok(holyday) => {
                    if em.delete {
                        self.delete_by_tag(&em.tag);
                    } else {
                        holyday.borrow_mut().modify(em);
                    }
                }
                Err(_e) => {
                    println!("tag {} not found, adding new holy day", &em.tag);
                    self.add(&em.to_holyday()?);
                }
            }
        }
        Ok(())
    }
    /** find the [Holyday] with a specified tag, or `None` */
    pub fn get_by_tag(&mut self, tag: &str) -> Result<HolydayRef, CalendarError> {
        let re = self.holydays_by_tag.get(tag);
        if let Some(r) = re {
            Ok(r.clone())
        } else {
            Err(CalendarError::new(&format!("unknown tag {}", tag)))
        }
    }
    /** Remove an [Holyday] from the [Calendar].

    The implementation is inefficient, but it should not be used very often. */
    pub fn delete_by_tag(&mut self, tag: &str) {
        if let Some(index) = self.holydays.iter().position(|e| e.borrow().tag == *tag) {
            self.holydays.swap_remove(index);
        }
        // if let  self.holydays.iter().position(|e| e.borrow().tag == *tag) {
        //     Some(index) => {
        //         self.holydays.swap_remove(index);
        //     }
        //     None => {}
        // }
        let _vo = self.holydays_by_tag.remove(tag);
    }
    /** get all holy days for this [Calendar], in order (Principal holy days first,...). */
    pub fn get_holydays(&self) -> Vec<HolydayRef> {
        let mut ee = self.holydays.clone();
        ee.sort();
        ee
    }
    // pub fn report_by_date(&self) {
    //     println!("{}", Green.on(Purple).bold().paint("holyday report by date"));
    //     let holydays = self.holydays.clone();
    //     holydays.sort_by(|a, b| Holyday::cmp_by_date(&a.borrow(), &b.borrow()));
    //     let mut last_date: Option<DateCal> = None;
    //     for e in &holydays {
    //         if Some(e.borrow().date_cal) != last_date {
    //             println!("{}", Green.bold().paint(format!("{:?}", e.date_cal)));
    //             last_date = Some(e.borrow().date_cal);
    //         }
    //         println!(
    //             "{} {}: {} {}",
    //             Blue.bold().paint(format!("{}", e.borrow().tag)),
    //             Yellow.paint(format!("{:?}", e.borrow().province)),
    //             e.borrow().title,
    //             e.borrow().source
    //         );
    //     }
    // }
}
/** An Holy Day is an holy day in a [Calendar] e.g. the holy days of the Anglican
Church of Hong Kong include Easter Sunday and Matteo Ricci.*/
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Holyday {
    /** the name of the holy day */
    pub title: String,
    /** description of an holy day */
    pub description: String,
    /** main attributes of commemorated e.g. martyr, bishop */
    pub main: HashSet<MainAttribute>,
    /** other attributes of commemorated e.g. spiritual guide */
    pub other: Vec<String>,
    /** death date of commemorated e.g. 379, 1833, c.269 */
    pub death: String,
    /** references on the internet to the holy day */
    pub refs: Vec<Reference>,
    /** the level of the holy day (commemoration, lesser festival,
    festival, principal feast, also unclassified) */
    pub class: HolydayClass,
    /** a short tag for identifying the holy day so that it can be
    overridden */
    pub tag: String,
    /** holy day has eve (and eve is not an specified holy day in its
    own right) */
    pub has_eve: bool,
    /** date calculation */
    pub date_cal: DateCal,
    /** whether and how the holy day can be transferred to another date */
    pub transfer: TransferType,
}
impl Holyday {
    /** modify an Holy Day according to an HolydayMod */
    pub fn modify(&mut self, m: &HolydayMod) {
        if let Some(t) = &m.title {
            self.title = t.to_string();
        }
        if let Some(mn) = &m.main {
            self.main = mn.clone();
        }
        if let Some(o) = &m.other {
            self.other = o.clone();
        }
        if let Some(d) = &m.death {
            self.death = d.to_string();
        }
        if let Some(rr) = &m.refs {
            self.refs = rr.clone();
        }
        if let Some(c) = &m.class {
            self.class = *c;
        }
        if let Some(e) = &m.has_eve {
            self.has_eve = *e;
        }
        if let Some(c) = &m.date_cal {
            self.date_cal = c.clone();
        }
        if let Some(t) = &m.transfer {
            self.transfer = *t;
        }
    }
    // fn cmp_by_date(a: &Self, b: &Self) -> Ordering {
    //     let c = a.date_cal.cmp(&b.date_cal);
    //     if c == Ordering::Equal {
    //         a.tag.cmp(&b.tag)
    //     } else {
    //         c
    //     }
    // }
}
impl Ord for Holyday {
    fn cmp(&self, other: &Self) -> Ordering {
        let cmp_class = self.class.cmp(&other.class);
        if cmp_class == Ordering::Equal {
            self.date_cal.cmp(&other.date_cal)
        } else {
            cmp_class.reverse()
        }
    }
}

impl PartialOrd for Holyday {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Default for Holyday {
    fn default() -> Self {
        Self {
            title: "".to_string(),
            description: "".to_string(),
            main: HashSet::new(),
            other: vec![],
            death: "".to_string(),
            refs: vec![],
            class: HolydayClass::Commemoration,
            tag: "".to_string(),
            has_eve: false,
            date_cal: DateCal::Fixed { month: 1, day: 1 },
            transfer: TransferType::Normal,
            // delete: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
/** EdMods is a set of edit changes to an [Calendar]. */
pub struct EdMods {
    /** the [EdMod]s in this EdMods */
    pub holydays: Vec<HolydayMod>,
}
impl EdMods {
    /** read edit mods from a reader */
    pub fn read<R>(reader: R) -> Result<Self, CalendarError>
    where
        R: io::Read,
    {
        println!("{}", Green.paint("reading edits"));
        let u: Self = from_reader(reader).map_err(CalendarError::from_error)?;
        println!("{}", Green.paint("read from reader done"));
        Ok(u)
    }
}
impl From<&mut Calendar> for EdMods {
    fn from(c: &mut Calendar) -> Self {
        Self {
            holydays: c
                .holydays
                .iter_mut()
                .map(|e| HolydayMod::from(e.borrow_mut().clone()))
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
/** A change to an [Holyday] or a new Holy Day. */
#[serde(default)]
pub struct HolydayMod {
    /** the name of the holyday */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /** a description of an [Holyday] */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /** main attributes of commemorated e.g. martyr, bishop */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main: Option<HashSet<MainAttribute>>,
    /** other attributes of commemorated e.g. spiritual guide */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other: Option<Vec<String>>,
    /** death date of commemorated e.g. 379, 1833, c.269 */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub death: Option<String>,
    /** references on the internet to the holy day */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refs: Option<Vec<Reference>>,
    /** the level of the holy day (commemoration, lesser festival,
    festival, principal feast, also unclassified) */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<HolydayClass>,
    /** a short tag for identifying the holy day so that it can be
    overridden */
    pub tag: String,
    /** holy day has eve (and eve is not an specified holy day in its
    own right) */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_eve: Option<bool>,
    /** date calculation */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_cal: Option<DateCal>,
    /** whether and how the holy day can be transferred to another date */
    pub transfer: Option<TransferType>,
    /** whether to delete the [Holyday] */
    pub delete: bool,
}
impl HolydayMod {
    /** convert an EdMod to an [Holyday]. All fields must be specified. */
    pub fn to_holyday(&self) -> Result<Holyday, CalendarError> {
        let e = Holyday {
            title: self
                .clone()
                .title
                .ok_or_else(|| CalendarError::new(&format!("no title ({:?})", self.tag)))?,
            description: self
                .clone()
                .description
                .ok_or_else(|| CalendarError::new("no description"))?,
            main: self
                .main
                .clone()
                .ok_or_else(|| CalendarError::new("no main"))?,
            other: self
                .other
                .clone()
                .ok_or_else(|| CalendarError::new("no other "))?,
            death: self
                .death
                .clone()
                .ok_or_else(|| CalendarError::new("no death "))?,
            refs: self
                .refs
                .clone()
                .ok_or_else(|| CalendarError::new("no refs"))?,
            class: self
                .class
                //    .clone()
                .ok_or_else(|| CalendarError::new("no class "))?,
            tag: self.tag.clone(),
            has_eve: self
                .has_eve
                .ok_or_else(|| CalendarError::new("no has_eve"))?,
            date_cal: self
                .date_cal
                .clone()
                .ok_or_else(|| CalendarError::new("no date_cal "))?,
            transfer: self
                .transfer
                .clone()
                .ok_or_else(|| CalendarError::new("no transfer "))?,
        };
        Ok(e)
    }
}
impl Default for HolydayMod {
    fn default() -> Self {
        Self {
            title: None,
            description: None,
            main: None,
            other: None,
            death: None,
            refs: None,
            class: None,
            tag: "".to_string(),
            has_eve: None,
            date_cal: None,
            transfer: None,
            delete: false,
        }
    }
}
impl From<Holyday> for HolydayMod {
    fn from(e: Holyday) -> Self {
        Self {
            title: Some(e.title),
            description: Some(e.description),
            main: Some(e.main),
            other: Some(e.other),
            death: Some(e.death),
            refs: Some(e.refs),
            class: Some(e.class),
            tag: e.tag,
            has_eve: Some(e.has_eve),
            date_cal: Some(e.date_cal),
            transfer: Some(e.transfer),
            delete: false,
        }
    }
}

/** Holy DayClass is the level of the holy day and can be commemoration,
lesser festival, festival, principal feast, also unclassified */
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub enum HolydayClass {
    NotAFestival,
    Unclassified,
    Commemoration,
    LesserFestival,
    Festival,
    CorpusChristi,
    Principal,
}
/** DateCal is instructions to calculate a date e.g. 25 Dec, 2 days before Easter Sunday. */
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub enum DateCal {
    /** Easter Sunday */
    Easter,
    /** Advent at the start of this Church year, so in the previous calendar year */
    Advent,
    /**  Advent at the start of the next Church year, so in this calendar year */
    AdventNext,
    /** relative to another date, such as Easter, Advent or a fixed date
    -- dates relative to Pentecost or Trinity are relative to Easter
    "on which the rest depend". For dates before the specified date,
    use a negative number for 'rel' */
    After { date: Box<DateCal>, rel: i16 },
    /** a specified day of the week after a specified date
    (e.g. Sunday after Epiphany) */
    Next {
        date: Box<DateCal>,
        day_of_week: OrderableDayOfWeek,
    },
    /** a date specified by month and day; may be in the previous
    calendar year (depending on the date relative to Advent). */
    Fixed { month: u8, day: u8 },
}
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
/** a [chrono::Weekday] with an ordering, so it can be part of a
sortable object. The actual order does not matter. */
pub struct OrderableDayOfWeek {
    /** the underlying day of the week */
    pub wd: chrono::Weekday,
}
impl Ord for OrderableDayOfWeek {
    fn cmp(&self, other: &Self) -> Ordering {
        self.wd
            .num_days_from_sunday()
            .cmp(&other.wd.num_days_from_sunday())
    }
}

impl PartialOrd for OrderableDayOfWeek {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl From<chrono::Weekday> for OrderableDayOfWeek {
    fn from(wd: chrono::Weekday) -> Self {
        Self { wd: wd }
    }
}
impl From<OrderableDayOfWeek> for chrono::Weekday {
    fn from(odw: OrderableDayOfWeek) -> Self {
        odw.wd
    }
}

/** TransferType indicates whether and how an holy day can be transferred
to another date */
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Copy, Clone)]
pub enum TransferType {
    /** follow the usual rules using the holy day's [HolydayClass] */
    Normal,
    /** special rule for Annunciation  */
    Annunciation,
    //   BaptismOfChrist,
    /** special rule for Joseph */
    Joseph,
    /** special rule for George */
    George,
    /** special rule for Mark  */
    Mark,
}
/** MainAttribute  */
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum MainAttribute {
    Martyr,
}
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
/** A Reference is a web page that is relevant to an [Holyday].
```
        use anglican_calendar::calendar::{Reference, WebSite};
        let r = Reference {
            website: WebSite::Wikipedia,
            article: "List_of_Anglican_Church_calendars".to_string(),
            description: "list of calendars".to_string(),
        };
        assert_eq!(
            "en.wikipedia.org/wiki/List_of_Anglican_Church_calendars",
            r.url()
        );
``` */
pub struct Reference {
    /** the web site */
    pub website: WebSite,
    /** the article within the web site */
    pub article: String,
    /** additional description text */
    pub description: String,
}
impl Reference {
    /** create a new Reference */
    pub fn new(website: WebSite, article: String) -> Self {
        Self {
            website,
            article: article.clone(),
            description: article.clone(),
        }
    }
    /** the URL for the Reference */
    pub fn url(self) -> String {
        self.website.prefix() + &self.article
    }
}
/** WebSite is a web site that contains relevant information.  */
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Copy, Clone)]
pub enum WebSite {
    Wikipedia,
}
impl WebSite {
    /** prefix is the prefix to a URL for this web site */
    pub fn prefix(self) -> String {
        match self {
            WebSite::Wikipedia => "en.wikipedia.org/wiki/".to_string(),
        }
    }
}
/** Season of the Church year */
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Copy, Clone)]
pub enum Season {
    Advent,
    Christmas,
    Epiphany,
    Lent,
    Easter,
    Ordinary,
}
/** A CalendarError is an [Error] which can be used in this crate.  */
#[derive(Debug, Clone)]
pub struct CalendarError {
    msg: String,
}
impl CalendarError {
    /** a CalendarError with the specified error text */
    pub fn new(m: &str) -> Self {
        CalendarError { msg: m.to_string() }
    }
    /** convert an [Error] of any type to CalendarError */
    pub fn from_error<Err>(err: Err) -> Self
    where
        Err: Error,
    {
        let srce = if let Some(s) = err.source() {
            format!(" from {}", s.description())
        } else {
            "".to_string()
        };
        println!("{}", Purple.bold().paint(format!("error is {:#?}", err)));
        CalendarError {
            msg: format!("error: {}{}", err.description().to_owned(), srce),
        }
    }
}
impl Error for CalendarError {
    fn description(&self) -> &str {
        &self.msg
    }
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
impl fmt::Display for CalendarError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error in calendar {}", self.msg)
    }
}
// #[cfg(test)]
// mod tests {
//    use super::*;
// #[test]
// /// test references
// fn test_references() {
//     let r = Reference {
//         website: WebSite::Wikipedia,
//         article: "List_of_Anglican_Church_calendars".to_string(),
//         description: "list of calendars".to_string(),
//     };
//     assert_eq!(
//         "en.wikipedia.org/wiki/List_of_Anglican_Church_calendar",
//         r.url()
//     );
// }
// }
