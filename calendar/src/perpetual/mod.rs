/*! Implements the year-independent data for a calendar. */

use ansi_term::Colour::*;
use chrono::{Local, Utc, Weekday};
use delegate::delegate;
use log::debug;
use serde_derive::{Deserialize, Serialize};
use std::{
    // borrow::{Borrow, BorrowMut},
    cell::RefCell,
    cmp::Ordering,
    collections::{hash_map::Keys, HashMap, HashSet},
    error::Error,
    fmt,
    hash::{Hash, Hasher},
    rc::Rc,
    str::FromStr,
};
use strum_macros::Display;
pub mod from_spreadsheet;
pub mod to_spreadsheet;
/// a value or an error code
pub type Result<T> = std::result::Result<T, CalendarError>;
use databake::{Bake, CrateEnv};
pub mod calendar;
pub mod holyday;
pub use holyday::Holyday;

/// wrap local date time so we can bake it
#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy, Serialize, Deserialize, Ord, PartialOrd)]
pub struct CalDateTime(chrono::DateTime<Local>);
impl Bake for CalDateTime {
    fn bake(&self, _ctx: &CrateEnv) -> databake::TokenStream {
        format!(
            "chrono::DateTime::from_timestamp({},0).expect(\"bad date\").into()",
            self.0.timestamp()
        )
        .parse()
        .unwrap()
    }
}
impl From<chrono::DateTime<Local>> for CalDateTime {
    fn from(date_time: chrono::DateTime<Local>) -> Self { Self(date_time) }
}
impl From<CalDateTime> for chrono::DateTime<Local> {
    fn from(cal_date_time: CalDateTime) -> Self { cal_date_time.0 }
}
impl From<chrono::DateTime<Utc>> for CalDateTime {
    fn from(date_time: chrono::DateTime<Utc>) -> Self { Self(date_time.with_timezone(&Local)) }
}
impl fmt::Display for CalDateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}
/** Information about a file that can be used e.g. for tracking its origin. */
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Bake)]
#[databake(path = calendar::calendar)]
pub struct FileInfo {
    pub description: String,
    pub created: CalDateTime,
    pub creation: String,
}
//const VERBOSE: bool = true;
/// whatever has a calendar
#[derive(
    Eq,
    PartialEq,
    Hash,
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    Ord,
    PartialOrd,
    strum::EnumString,
    strum::VariantNames,
    Bake,
)]
#[databake(path = calendar::calendar)]
pub enum Province {
    ChurchOfEngland,
    HongKong,
    ECUSA,
    Australia,
    SouthAfrica,
    Canada,
    BCP,
    Unknown,
    All,
}
// impl Province {
//     fn from_short_str(s: &str) -> Result<Self, CalendarError> {
//         match s {
//             "cofe" | "en" => Ok(Province::ChurchOfEngland),
//             "hkskh" | "hk" => Ok(Province::HongKong),
//             "ecusa" | "tec" | "usa" | "us" => Ok(Province::ECUSA),
//             "aca" | "au" => Ok(Province::Australia),
//             "acsa" | "sa" => Ok(Province::SouthAfrica),
//             "acc" | "ca" => Ok(Province::Canada),
//             "bcp" => Ok(Province::BCP),
//             "all" => Ok(Province::All),
//             _ => Err(CalendarError::new(&format!("unknown province {}",
// &s))),         }
//     }
// }
impl fmt::Display for Province {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Province::ChurchOfEngland => write!(f, "Church of England"),
            Province::HongKong => write!(f, "Hong Kong SKH"),
            Province::ECUSA => write!(f, "Episcopal Church of USA"),
            Province::Australia => write!(f, "Anglican Church of Australia"),
            Province::SouthAfrica => write!(f, "Anglican Church of South Africa"),
            Province::Canada => write!(f, "Anglican Church of Canada"),
            Province::BCP => write!(f, "Book of Common Prayer 1662"),
            Province::Unknown => write!(f, "Unknown"),
            Province::All => write!(f, "Combined Calendar"),
        }
    }
}
#[cfg(test)]
/** Data about a [Province] */
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvinceData {
    /** the province */
    pub province: Province,
    /** the abbreviation for the province, also used in file names */
    pub abbrev: String,
}
#[cfg(test)]
impl ProvinceData {
    // used in tests
    fn make(province: Province, abbrev: &str) -> Self {
        Self {
            province,
            abbrev: abbrev.to_string(),
        }
    }
}
/** List of all [Province]s and their [ProvinceData]

```
    use calendar::calendar::{ProvinceList, Province};
    use std::str::FromStr;

    let prov_list = ProvinceList::make();
    for (prov,prov_data) in prov_list.all() {
        assert_eq!(prov_data, prov_list.get(*prov));
        assert_eq!(*prov,prov_data.province);
        assert_eq!(Province::from_str(&prov_data.abbrev).unwrap(), *prov);
    }
```
*/
#[cfg(test)]
pub struct ProvinceList {
    provinces: HashMap<Province, ProvinceData>,
}
#[cfg(test)]
impl ProvinceList {
    fn add(&mut self, province: Province, abbrev: &str) {
        self.provinces
            .insert(province, ProvinceData::make(province, abbrev));
    }

    /** returns the [ProvinceList] */
    pub fn make() -> Self {
        let mut pl = Self {
            provinces: HashMap::new(),
        };
        pl.add(Province::ChurchOfEngland, "cofe");
        pl.add(Province::HongKong, "hkskh");
        pl.add(Province::ECUSA, "ecusa");
        pl.add(Province::Australia, "aca");
        pl.add(Province::SouthAfrica, "acsa");
        pl.add(Province::Canada, "acc");
        pl.add(Province::BCP, "bcp");
        pl.add(Province::All, "all");
        pl
    }

    /** gets the data for one [Province] */
    pub fn get(&self, province: Province) -> Result<&ProvinceData> {
        self.provinces.get(&province).ok_or(CalendarError {
            msg: "unknown province".to_string(),
        })
    }

    /** all the provinces */
    pub fn all(&self) -> &HashMap<Province, ProvinceData> { &self.provinces }
}
impl FileInfo {
    /** create a FileInfo with the specified values (really used) */
    pub fn new(description: &str, creation: &str) -> Self {
        Self {
            description: description.to_string(),
            created: chrono::Local::now().into(),
            creation: creation.to_string(),
        }
    }

    /** set the creation string */
    pub fn set_creation(&mut self, creation: &str) { self.creation = creation.to_string() }
}
impl Default for FileInfo {
    fn default() -> Self {
        Self {
            description: "".to_string(),
            created: chrono::Local::now().into(),
            creation: "".to_string(),
        }
    }
}
/** A reference-counted pointer to an [Holyday]
An `HolydayRef` reference to a Holyday */
#[derive(PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct HolydayRef {
    r: Rc<RefCell<Holyday>>,
}
impl Bake for HolydayRef {
    fn bake(&self, ctx: &CrateEnv) -> databake::TokenStream {
        format!("HolydayRef::new({})", self.r.borrow().bake(ctx))
            .parse()
            .unwrap()
    }
}
impl HolydayRef {
    // /** modify an Holy Day according to an HolydayMod */
    // pub fn modify(&mut self, m: &HolydayMod) {
    //     self.r.as_ref().borrow_mut().modify(m);
    // }
    delegate! {
        to self.r.as_ref().borrow_mut() {
            // /** modify an Holy Day according to an HolydayMod */
            // pub fn modify(&mut self, m: &HolydayMod);
            /** `clean_up` tidies up a [Holyday] */
            pub fn clean_up(&mut self);
        }
    }

    delegate! {
        to self.r.as_ref().borrow() {
            /* holy day has eve (and eve is not an specified holy day in its own right)  */
            pub fn has_eve(&self) -> bool;
            /** `class` returns the class of the holyday */
            pub fn class(&self) -> HolydayClass ;  /** `transfer` returns the transfer type of the holyday */
            pub fn transfer(&self) -> TransferType ;
        }
    }
}

impl HolydayRef {
    /** Create a new HolydayRef */
    pub fn new(hd: Holyday) -> Self {
        Self {
            r: Rc::new(RefCell::new(hd)),
        }
    }

    /** compare holydays using just the date calc */
    fn cmp_by_date_cal(&self, other: &Self) -> Ordering {
        Holyday::from(self.clone()).cmp_by_date_cal(&Holyday::from(other.clone()))
    }

    /** compare holydays using just the tag */
    fn cmp_by_tag(&self, other: &Self) -> Ordering {
        Holyday::from(self.clone()).cmp_by_tag(&Holyday::from(other.clone()))
    }

    /** with_inner runs a closure against the Holyday. */
    pub fn with_inner<T>(&self, f: impl FnOnce(&Holyday) -> T) -> T {
        let hr: &Holyday = &self.r.as_ref().borrow();
        f(hr)
    }

    /** `tag` returns the tag of the holyday */
    pub fn tag(&self) -> String {
        let hr: &Holyday = &self.r.as_ref().borrow();
        hr.tag.clone()
    }

    /** `date_cal` returns the date_cal of the holyday */
    pub fn date_cal(&self) -> DateCal {
        let hr: &Holyday = &self.r.as_ref().borrow();
        hr.date_cal.clone()
    }

    /** `title` returns the title of the holyday */
    pub fn title(&self) -> String {
        let hr: &Holyday = &self.r.as_ref().borrow();
        hr.title.clone()
    }

    // /** `transfer` returns the transfer type of the holyday */
    // pub fn transfer(&self) -> TransferType {
    //     let hr: &Holyday = &self.r.as_ref().borrow();
    //     hr.transfer.clone()
    // }

    /** `main` returns the main of the holyday */
    pub fn main(&self) -> HashSet<MainAttribute> {
        let hr: &Holyday = &self.r.as_ref().borrow();
        hr.main.clone()
    }

    /** `description` returns the description of the holyday */
    pub fn description(&self) -> String {
        let hr: &Holyday = &self.r.as_ref().borrow();
        hr.description.clone()
    }

    /** `refs` returns the refs of the holyday */
    pub fn refs(&self) -> Vec<Reference> {
        let hr: &Holyday = &self.r.as_ref().borrow();
        hr.refs.clone()
    }

    /** `other` returns the other of the holyday */
    pub fn other(&self) -> Vec<String> {
        let hr: &Holyday = &self.r.as_ref().borrow();
        hr.other.clone()
    }

    /** `death` returns death date of commemorated e.g. 379, 1833, c.269 */
    pub fn death(&self) -> String {
        let hr: &Holyday = &self.r.as_ref().borrow();
        hr.death.clone()
    }
}
impl Hash for HolydayRef {
    fn hash<H: Hasher>(&self, state: &mut H) { self.r.as_ref().borrow_mut().hash(state); }
}

impl fmt::Debug for HolydayRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{:?}", self.r.as_ref()) }
}
// impl Clone for HolydayRef {
//     fn clone(&self) -> Self {
//         Self { r: self.r.clone() }
//     }
// }
// impl<'a> Into<&'a Holyday> for HolydayRef {
//     fn into(self) -> &'a Holyday {
//         &self.r.as_ref().borrow()
//     }
// }
/** An `HolydaySort` is an ordering of Holydays within a calendar file. None of
these orderings will give chronological order because of moveable holydays. */
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Display)]
pub enum HolydaySort {
    /// do not Sort
    NoSort,
    /// order by class and then date_cal (Ord above)
    Normal,
    /// order by date calculation
    DateCal,
    /// order by tag
    Tag,
}
impl Default for HolydaySort {
    fn default() -> Self { Self::NoSort }
}
impl FromStr for HolydaySort {
    type Err = CalendarError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "nosort" => Ok(HolydaySort::NoSort),
            "normal" => Ok(HolydaySort::Normal),
            "datecal" => Ok(HolydaySort::DateCal),
            "tag" => Ok(HolydaySort::Tag),
            _x => Err(CalendarError::new(&format!("bad sort {}", _x))),
        }
    }
}

// #[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
// /** EdMods is a set of edit changes to an [Calendar]. */
// pub struct EdMods {
//     #[serde(default)]
//     info: FileInfo,
//     /** the [HolydayMod]s in this EdMods */
//     pub holydays: Vec<HolydayMod>,
// }
// impl EdMods {
//     /** read edit mods from a reader */
//     pub fn read<R>(reader: R) -> Result<Self>
//     where
//         R: io::Read,
//     {
//         println!("{}", Green.paint("reading edits"));
//         let u: Self =
// from_reader(reader).map_err(CalendarError::from_error)?;         println!(
//             "{}",
//             Green.paint(format!("modifications read from reader with {:?}",
// u.info))         );
//         Ok(u)
//     }
// }
// impl From<&mut Calendar> for EdMods {
//     fn from(c: &mut Calendar) -> Self {
//         Self {
//             info: c.info.clone(),
//             holydays: c
//                 .holydays
//                 .iter_mut()
//                 .map(|e| {
//                     let e: Holyday = e.clone().into();
//                     HolydayMod::from(e)
//                 })
//                 .collect(),
//         }
//     }
// }

// #[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
// /** A change to an [Holyday] or a new Holy Day. */
// #[serde(default)]
// pub struct HolydayMod {
//     /** the name of the holyday */
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub title: Option<String>,
//     /** a description of an [Holyday] */
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub description: Option<String>,
//     /** main attributes of commemorated e.g. martyr, bishop */
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub main: Option<HashSet<MainAttribute>>,
//     /** other attributes of commemorated e.g. spiritual guide */
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub other: Option<Vec<String>>,
//     /** death date of commemorated e.g. 379, 1833, c.269 */
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub death: Option<String>,
//     /** references on the internet to the holy day */
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub refs: Option<Vec<Reference>>,
//     /** the level of the holy day (commemoration, lesser festival,
//     festival, principal feast, also unclassified) */
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub class: Option<HolydayClass>,
//     /** a short tag for identifying the holy day so that it can be
//     overridden */
//     pub tag: String,
//     /** holy day has eve (and eve is not an specified holy day in its
//     own right) */
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub has_eve: Option<bool>,
//     /** date calculation */
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub date_cal: Option<DateCal>,
//     /** whether and how the holy day can be transferred to another date */
//     pub transfer: Option<TransferType>,
//     /** whether to delete the [Holyday] */
//     pub delete: bool,
// }
// impl HolydayMod {
//     /** convert an EdMod to an [Holyday]. All fields must be specified. */
//     pub fn to_holyday(&self) -> Result<Holyday> {
//         let title = self.clone().title.ok_or_else(|| {
//             CalendarError::new(&format!(
//                 "adding holy day and field not specified - title, check that
// the edit tag ({:?}) \                  is found in a calendar",
//                 self.tag
//             ))
//         })?;
//         let e = Holyday {
//             title: title.clone(),
//             description: self.clone().description.unwrap_or(title.clone()),
//             main: self.main.clone().ok_or_else(|| {
//                 CalendarError::new("adding holy day and field not specified -
// main")             })?,
//             other: self.other.clone().ok_or_else(|| {
//                 CalendarError::new("adding holy day and field not specified -
// other ")             })?,
//             death: self.death.clone().unwrap_or("".to_string()),
//             refs: self.refs.clone().ok_or_else(|| {
//                 CalendarError::new("adding holy day and field not specified -
// refs")             })?,
//             class: self
//                 .class
//                 //    .clone()
//                 .ok_or_else(|| {
//                     CalendarError::new("adding holy day and field not
// specified -  class ")                 })?,
//             tag: self.tag.clone(),
//             has_eve: self.has_eve.ok_or_else(|| {
//                 CalendarError::new("adding holy day and field not specified -
// has_eve")             })?,
//             date_cal: self
//                 .date_cal
//                 .clone() // igadding holy day and field not specified - re
// clippy                 .ok_or_else(|| {
//                     CalendarError::new("adding holy day and field not
// specified -  date_cal ")                 })?,
//             transfer: self.transfer.ok_or_else(|| {
//                 CalendarError::new("adding holy day and field not specified -
// transfer ")             })?,
//         };
//         Ok(e)
//     }
// }
// impl Default for HolydayMod {
//     fn default() -> Self {
//         Self {
//             title: None,
//             description: None,
//             main: None,
//             other: None,
//             death: None,
//             refs: None,
//             class: None,
//             tag: "".to_string(),
//             has_eve: None,
//             date_cal: None,
//             transfer: None,
//             delete: false,
//         }
//     }
// }
// impl From<Holyday> for HolydayMod {
//     fn from(e: Holyday) -> Self {
//         Self {
//             title: some_unless_blank(&e.title),
//             description: some_unless_blank(&e.description),
//             main: Some(e.main),
//             other: Some(e.other),
//             death: some_unless_blank(&e.death),
//             refs: Some(e.refs),
//             class: Some(e.class),
//             tag: e.tag,
//             has_eve: Some(e.has_eve),
//             date_cal: Some(e.date_cal),
//             transfer: Some(e.transfer),
//             delete: false,
//         }
//     }
// }
/// returns None for an empty string, otherwise Some
fn some_unless_blank(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

/** Holy DayClass is the level of the holy day and can be commemoration, lesser
festival, festival, principal feast, also unclassified and (ordinary) Sunday*/
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Copy,
    Clone,
    strum::EnumString,
    strum::VariantNames,
    Bake,
)]
#[databake(path = calendar::calendar)]
#[strum(ascii_case_insensitive)]
pub enum HolydayClass {
    NotAFestival,
    Unclassified,
    Commemoration,
    LesserFestival,
    Festival,
    Sunday,
    CorpusChristi,
    Principal,
}
impl fmt::Display for HolydayClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Debug::fmt(self, f) }
}
/** DateCal is an instruction to calculate a date e.g. 25 Dec, 2 days before
Easter Sunday. */
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Clone,
    Hash,
    strum::Display,
    strum::VariantNames,
    strum::EnumDiscriminants,
    strum::EnumCount,
    Bake,
)]
#[databake(path = calendar::calendar)]
#[strum_discriminants(derive(strum::VariantArray))]
pub enum DateCal {
    /** Easter Sunday */
    Easter,
    /** Advent at the start of this Church year, so in the previous calendar
     * year */
    Advent,
    /** Advent at the start of the next Church year, so in this calendar year */
    AdventNext,
    /** relative to another date, such as Easter, Advent or a fixed date
    -- dates relative to Pentecost or Trinity are relative to Easter
    "on which the rest depend". For dates before the specified date,
    use a negative number for 'rel' */
    After { date: Box<DateCal>, rel: i16 },
    /** obsolete, do not use */
    Next {
        date: Box<DateCal>,
        day_of_week: OrderableDayOfWeek,
    },
    /** the Sunday after a specified date (e.g. Sunday after Epiphany) */
    NextSunday { date: Box<DateCal> },
    /** a date specified by month and day; may be in the previous
    calendar year (depending on the date relative to Advent). */
    Fixed { month: u8, day: u8 },
}
impl DateCal {
    /** `new_from_strings` creates a [DateCal from 3 strings] TODO better
     * error handling */
    pub fn new_from_strings(kind: &str, v1: &str, v2: &str) -> Result<Self> {
        debug!("date cal from ({kind}/{v1}/{v2})");
        Ok(match kind {
            "Easter" => DateCal::Easter,
            "Advent" => DateCal::Advent,
            "AdventNext" => DateCal::AdventNext,
            "After" => DateCal::After {
                date: Box::new(Self::new_from_strings(v1, "", "")?),
                rel: v2.parse().map_err(CalendarError::from_error)?,
            }, // TODO fix error handling
            "Next" => DateCal::NextSunday {
                // obsolete, convert
                date: Box::new(Self::new_from_strings("Fixed", v1, v2)?),
            },
            "NextSunday" => DateCal::NextSunday {
                date: Box::new(Self::new_from_strings("Fixed", v1, v2)?),
            },
            "Fixed" => DateCal::Fixed {
                month: v1.parse::<u8>().unwrap_or_default(),
                day: v2.parse::<u8>().unwrap_or_default(),
            },
            _ => {
                return Err(CalendarError::new(&format!(
                    "unknown date calculation: {kind}"
                )))
            },
        })
    }

    /** `try_month` gets the month of the date if that makes sense */
    pub fn try_month_and_day(&self) -> Result<(u8, u8)> {
        match self {
            DateCal::Next {
                date,
                day_of_week: _,
            } => date.try_month_and_day(),
            DateCal::NextSunday { date } => date.try_month_and_day(),
            DateCal::Fixed { month, day } => Ok((*month, *day)),
            _ => Err(CalendarError::new("cannot get month and day here")),
        }
    }

    /** `fixed` value as cleaned up */
    pub fn fixed(&self) -> Self {
        match self {
            DateCal::Next {
                date,
                day_of_week: _,
            } => DateCal::NextSunday { date: date.clone() },
            _ => self.clone(),
        }
    }
}
impl DateCalDiscriminants {
    /** `description` is the description of a [DateCal] */
    pub fn description(&self) -> String {
        match self {
            Self::Easter => "Easter Sunday",
            Self::Advent => {
                "Advent at the start of this Church year, so in the previous calendar year"
            },
            Self::AdventNext => {
                "Advent at the start of the next Church year, so in this calendar year "
            },
            Self::After => "relative to another date, e.g. to Easter; negative means before",
            /*a specified day of the week after a specified date
            (e.g. Sunday after Epiphany) */
            Self::Next => "(do not use)",
            Self::NextSunday => "a Sunday after a specified date (e.g. after Epiphany)",
            /* may be in the previous
            calendar year (depending on the date relative to Advent). */
            Self::Fixed => "specified by month and day",
        }
        .to_string()
    }
}
// #[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Hash)]
// #[derive(Bake)]
// #[databake(path = calendar::calendar)]
/** a [chrono::Weekday] with an ordering, so it can be part of a
sortable object. The actual order does not matter. */
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Hash)]
pub struct OrderableDayOfWeek {
    /** the underlying day of the week */
    pub wd: chrono::Weekday,
}
impl OrderableDayOfWeek {
    /** `new_from_string` creates a a[OrderableDayOfWeek] from a string */
    pub fn new_from_str(s: &str) -> Result<Self> {
        Ok(Self {
            wd: Weekday::from_str(s).map_err(CalendarError::from_error)?,
        })
    }
}
impl Bake for OrderableDayOfWeek {
    fn bake(&self, _ctx: &CrateEnv) -> databake::TokenStream { self.to_string().parse().unwrap() }
}
impl Ord for OrderableDayOfWeek {
    fn cmp(&self, other: &Self) -> Ordering {
        self.wd
            .num_days_from_sunday()
            .cmp(&other.wd.num_days_from_sunday())
    }
}

impl PartialOrd for OrderableDayOfWeek {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}
impl From<chrono::Weekday> for OrderableDayOfWeek {
    fn from(wd: chrono::Weekday) -> Self { Self { wd } }
}
impl From<OrderableDayOfWeek> for chrono::Weekday {
    fn from(odw: OrderableDayOfWeek) -> Self { odw.wd }
}
impl fmt::Display for OrderableDayOfWeek {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.wd) }
}

/** TransferType indicates whether and how an holy day can be transferred
to another date or dropped */
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Eq,
    PartialEq,
    Clone,
    Copy,
    strum::Display,
    strum::EnumString,
    strum::VariantNames,
    strum::EnumDiscriminants,
    strum::EnumCount,
    Bake,
)]
#[databake(path = calendar::calendar)]
#[strum(ascii_case_insensitive)]
#[strum_discriminants(derive(strum::VariantArray))]
pub enum TransferType {
    /** follow the usual rules using the holy day's [HolydayClass] */
    Normal,
    /** special rule for Annunciation */
    Annunciation,
    //   BaptismOfChrist,
    /** special rule for Joseph */
    Joseph,
    /** special rule for George */
    George,
    /** special rule for Mark */
    Mark,
    // /** must occur before Advent, otherwise drop */
    // BeforeAdventNext,
    /** do not transfer */
    DoNotTransfer,
    /** transfer to a nearby Sunday (if not a Sunday) for Epiphany (optional
    rule) */
    EpiphanyOption,
    /** transfer to a nearby Sunday (if not a Sunday) for
    Presentation/Candlemas (optional rule) */
    PresentationOption,
    /** transfer to a nearby Sunday (if not a Sunday) for All Saints'
    (optional rule) */
    AllSaintsOption,
    /** transfer next Sunday (if not a Sunday) */
    TransferNextSunday,
    /** do not keep if it coincides with a higher holy day */
    DropOnClash,
    /** transfer to the next available day if it coincides with a higher holy
    day */
    NextDayOnClash,
    /** keep if it coincides with a higher holy day (keep both) and downgrade */
    KeepOnClash,
}
impl Default for TransferType {
    fn default() -> Self { Self::Normal }
}
impl TransferType {
    pub fn new() -> Self { Self::Normal }
}
impl TransferTypeDiscriminants {
    /** `description` is the description of a [TransferType] */
    pub fn description(&self) -> String {
        match self {
            TransferTypeDiscriminants::Normal => {
                "follow the usual rules using the holy day's Class"
            },
            TransferTypeDiscriminants::Annunciation => "special rule for Annunciation",
            TransferTypeDiscriminants::Joseph => "special rule for Joseph",
            TransferTypeDiscriminants::George => "special rule for George",
            TransferTypeDiscriminants::Mark => "special rule for Mark ",
            TransferTypeDiscriminants::DoNotTransfer => "do not transfer",
            TransferTypeDiscriminants::EpiphanyOption => {
                "transfer to a nearby Sunday (if not a Sunday) for Epiphany (optional rule)"
            },
            TransferTypeDiscriminants::PresentationOption => {
                "transfer to a nearby Sunday (if not a Sunday) for Presentation/Candlemas \
                 (optional rule)"
            },
            TransferTypeDiscriminants::AllSaintsOption => {
                "transfer to a nearby Sunday (if not a Sunday) for All Saints' (optional rule) "
            },
            TransferTypeDiscriminants::TransferNextSunday => {
                "transfer next Sunday (if not a Sunday)"
            },
            TransferTypeDiscriminants::DropOnClash => {
                "do not keep if it coincides with a higher holy day"
            },
            TransferTypeDiscriminants::NextDayOnClash => {
                "transfer to the next available day if it coincides with a higher holy day"
            },
            TransferTypeDiscriminants::KeepOnClash => {
                "keep if it coincides with a higher holy day (keep both) and downgrade"
            },
        }
        .to_string()
    }
}
/** A principal attribute of a [Holyday] (at present only
 * [MainAttribute::Martyr]) */
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Eq,
    PartialEq,
    Copy,
    Clone,
    Hash,
    strum::Display,
    strum::EnumString,
    strum::VariantNames,
    Bake,
)]
#[databake(path = calendar::calendar)]
#[strum(ascii_case_insensitive)]
pub enum MainAttribute {
    Martyr,
}
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Hash, Bake)]
#[databake(path = calendar::calendar)]
/** A Reference is a web page that is relevant to an [Holyday].
```
        use calendar::perpetual::{Reference, WebSite};
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
    pub fn url(&self) -> String { (self.website.prefix() + &self.article).clone() }
}
/** WebSite is a web site that contains relevant information. */
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Copy, Clone, Hash, Bake)]
#[databake(path = calendar::calendar)]
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
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Copy, Clone, strum::VariantNames)]
pub enum Season {
    Advent,
    Christmas,
    Epiphany,
    Lent,
    Easter,
    Ordinary,
}
/** the colour for a [Holyday] */
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SeasonColour {
    White,
    Red,
    Purple,
    Green,
}
impl SeasonColour {
    /** colour for HTML */
    pub fn colour_a(&self) -> String {
        match self {
            SeasonColour::White => "white",
            SeasonColour::Red => "red",
            SeasonColour::Purple => "purple",
            SeasonColour::Green => "green",
        }
        .to_string()
    }

    /** colour for HTML */
    pub fn colour_b(&self) -> String {
        match self {
            SeasonColour::White => "black",
            _ => "white",
        }
        .to_string()
    }
}
impl fmt::Display for SeasonColour {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Debug::fmt(self, f) }
}
/** A CalendarError is an [Error] which can be used in this crate. */
#[derive(Debug, Clone)]
pub struct CalendarError {
    msg: String,
}
impl CalendarError {
    /** a CalendarError with the specified error text */
    pub fn new(m: &str) -> Self { CalendarError { msg: m.to_string() } }

    /** convert an [Error] of any type to CalendarError */
    pub fn from_error<Err>(err: Err) -> Self
    where
        Err: Error,
    {
        let srce = if let Some(s) = err.source() {
            format!(" from {:?}", s.source())
        } else {
            "".to_string()
        };
        println!("{}", Purple.bold().paint(format!("error is {:#?}", err)));
        CalendarError {
            msg: format!("error: {:?}{}", err.source().to_owned(), srce),
        }
    }
}
impl Error for CalendarError {
    fn description(&self) -> &str { &self.msg }

    fn source(&self) -> Option<&(dyn Error + 'static)> { None }
}
impl fmt::Display for CalendarError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error in calendar {}", self.msg)
    }
}
/** list all [ProvHolyday]s, grouped by date, for reports */
#[derive(Default, Debug)]
pub struct ProvHolydaysByDate {
    phbd: HashMap<DateCal, Vec<ProvHolyday>>,
}
impl ProvHolydaysByDate {
    /** load all the [Holyday]s from a [Calendar] */
    pub fn load_calendar(&mut self, cal: &calendar::Calendar) {
        for hd in cal.get_holydays() {
            let ph = ProvHolyday {
                province: cal.province,
                holyday: hd.clone(),
            };
            let dc = hd.date_cal();
            if let Some(vph) = self.phbd.get_mut(&dc) {
                vph.push(ph);
            } else {
                let nvph = vec![ph];
                self.phbd.insert(dc, nvph);
            }
        }
    }

    /** get all the dates with  [Holyday]s */
    pub fn dates(&self) -> Keys<DateCal, Vec<ProvHolyday>> { self.phbd.keys() }

    /** get the   [ProvHolyday]s for a date */
    pub fn by_date(&self, date: DateCal) -> impl Iterator<Item = &ProvHolyday> {
        self.phbd.get(&date).unwrap().iter()
    }
}
/** list all [ProvHolyday]s, grouped by tag, for reports */
#[derive(Default, Debug)]
pub struct ProvHolydaysByTag {
    phbt: HashMap<String, Vec<ProvHolyday>>,
}
impl ProvHolydaysByTag {
    /** load all the [Holyday]s from a [Calendar] */
    pub fn load_calendar(&mut self, cal: &calendar::Calendar) {
        for hd in cal.get_holydays() {
            let ph = ProvHolyday {
                province: cal.province,
                holyday: hd.clone(),
            };
            let tag = hd.tag();
            if let Some(vph) = self.phbt.get_mut(&tag) {
                vph.push(ph);
            } else {
                let nvph = vec![ph];
                self.phbt.insert(tag.to_string(), nvph);
            }
        }
    }

    /** get all the tags with  [Holyday]s */
    pub fn tags(&self) -> Keys<String, Vec<ProvHolyday>> { self.phbt.keys() }

    /** get the   [ProvHolyday]s for a tag */
    pub fn by_tag(&self, tag: String) -> impl Iterator<Item = &ProvHolyday> {
        self.phbt.get(&tag).unwrap().iter()
    }
}
/** a [Holyday] in a specific [Calendar] (identified by its [Province]) */
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Ord, PartialOrd)]
pub struct ProvHolyday {
    /** the province */
    pub province: Province,
    /** the holy day */
    pub holyday: HolydayRef,
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
