/*! Implements the year-independent data for a calendar.  */

use ansi_term::Colour::*;
use chrono::Utc;
use ron::de::from_reader;
use ron::ser::to_string_pretty;
use serde_derive::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::hash_map::Keys;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::io;
use std::rc::Rc;
use std::str::FromStr;
use strum_macros::{Display, EnumString};

/** A [Calendar] contains the [Holyday]s for a 'province' e.g. the Anglican
Church of Hong Kong. A Calendar is not specific to a specific year.*/
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Calendar {
    #[serde(default)]
    /** info about the file */
    pub info: FileInfo,
    /** the province owning this calendar */
    pub province: Province,
    holydays: Vec<HolydayRef>,
    #[serde(skip)]
    holydays_by_tag: HashMap<String, HolydayRef>,
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
    pub fn new() -> Self {
        Self::default()
    }
    /** add an [Holyday] to a [Calendar] */
    pub fn add(&mut self, holyday: &Holyday) {
        let r = HolydayRef::new(holyday.clone());
        self.holydays.push(r.clone());
        self.holydays_by_tag.insert(holyday.tag.clone(), r.clone());
    }
    /** read a calendar from a reader */
    pub fn read<R>(reader: R) -> Result<Self, CalendarError>
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
                Ok(mut holyday) => {
                    if em.delete {
                        self.delete_by_tag(&em.tag);
                    } else {
                        holyday.modify(em);
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
        if let Some(index) = self.holydays.iter().position(|e| e.tag() == *tag) {
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
    /** `sort` sorts the calendar into a consistent order (using Ord). */
    pub fn sort(&mut self) {
        self.holydays.sort();
    }
    /** `sort` sorts the calendar into a consistent order by date calculation. */
    pub fn sort_by_date_cal(&mut self) {
        self.holydays.sort_by(HolydayRef::cmp_by_date_cal);
    }
    /** `sort` sorts the calendar into a consistent order by tag. */
    pub fn sort_by_tag(&mut self) {
        self.holydays.sort_by(HolydayRef::cmp_by_tag);
    }
}
/** Information about a file that can be used e.g. for tracking its origin. */
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct FileInfo {
    description: String,
    created: chrono::DateTime<Utc>,
    creation: String,
}
//const VERBOSE: bool = true;
/// whatever has a calendar
#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy, Serialize, Deserialize, Ord, PartialOrd)]
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
impl FromStr for Province {
    type Err = CalendarError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cofe" | "en" => Ok(Province::ChurchOfEngland),
            "hkskh" | "hk" => Ok(Province::HongKong),
            "ecusa" | "tec" | "usa" | "us" => Ok(Province::ECUSA),
            "aca" | "au" => Ok(Province::Australia),
            "acsa" | "sa" => Ok(Province::SouthAfrica),
            "acc" | "ca" => Ok(Province::Canada),
            "bcp" => Ok(Province::BCP),
            "all" => Ok(Province::All),
            _ => Err(CalendarError::new(&format!("unknown province {}", &s))),
        }
    }
}
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
/** Data about a [Province] */
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvinceData {
    /** the province */
    pub province: Province,
    /** the abbreviation for the province, also used in file names */
    pub abbrev: String,
}
impl ProvinceData {
    fn make(province: Province, abbrev: &str) -> Self {
        Self {
            province,
            abbrev: abbrev.to_string(),
        }
    }
}
/** List of all [Province]s and their [ProvinceData]

```
    use anglican_calendar::calendar::{ProvinceList, Province};
    use std::str::FromStr;

    let prov_list = ProvinceList::make();
    for (prov,prov_data) in prov_list.all() {
        assert_eq!(prov_data, prov_list.get(*prov));
        assert_eq!(*prov,prov_data.province);
        assert_eq!(Province::from_str(&prov_data.abbrev).unwrap(), *prov);
    }
```
*/
pub struct ProvinceList {
    provinces: HashMap<Province, ProvinceData>,
}
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
    pub fn get(&self, province: Province) -> &ProvinceData {
        self.provinces.get(&province).unwrap()
    }
    /** all the provinces */
    pub fn all(&self) -> &HashMap<Province, ProvinceData> {
        &self.provinces
    }
}
impl FileInfo {
    /** create a FileInfo with the specified values */
    pub fn new(description: &str, creation: &str) -> Self {
        Self {
            description: description.to_string(),
            created: chrono::Utc::now(),
            creation: creation.to_string(),
        }
    }
    /** set the creation string */
    pub fn set_creation(&mut self, creation: &str) {
        self.creation = creation.to_string()
    }
}
impl Default for FileInfo {
    fn default() -> Self {
        Self {
            description: "".to_string(),
            created: chrono::Utc::now(),
            creation: "".to_string(),
        }
    }
}
/** An Holy Day is an holy day in a [Calendar] e.g. the holy days of the Anglican
Church of Hong Kong include Easter Sunday and Matteo Ricci.*/
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(default)]
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
    /** whether and how the holy day must be transferred to another
    date or dropped */
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
            self.transfer = t.clone();
        }
    }
    fn cmp_by_date_cal(&self, other: &Self) -> Ordering {
        self.date_cal.cmp(&other.date_cal)
    }
    fn cmp_by_tag(&self, other: &Self) -> Ordering {
        self.tag.cmp(&other.tag)
    }
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
        }
    }
}
impl From<HolydayRef> for Holyday {
    fn from(rf: HolydayRef) -> Self {
        rf.r.as_ref().borrow().clone()
    }
}
/** A reference-counted pointer to an [Holyday]
An `HolydayRef` reference to a Holyday */
#[derive(PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct HolydayRef {
    r: Rc<RefCell<Holyday>>,
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
    /** `class` returns the class of the holyday */
    pub fn class(&self) -> HolydayClass {
        let hr: &Holyday = &self.r.as_ref().borrow();
        hr.class
    }
    /** `transfer` returns the transfer type of the holyday */
    pub fn transfer(&self) -> TransferType {
        let hr: &Holyday = &self.r.as_ref().borrow();
        hr.transfer.clone()
    }
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
    /** modify an Holy Day according to an HolydayMod */
    pub fn modify(&mut self, m: &HolydayMod) {
        self.r.as_ref().borrow_mut().modify(m);
    }
}
impl fmt::Debug for HolydayRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.r.as_ref())
    }
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
    fn default() -> Self {
        Self::NoSort
    }
}
impl FromStr for HolydaySort {
    type Err = CalendarError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "nosort" => Ok(HolydaySort::NoSort),
            "normal" => Ok(HolydaySort::Normal),
            "datecal" => Ok(HolydaySort::DateCal),
            "tag" => Ok(HolydaySort::Tag),
            _x => Err(CalendarError::new(&format!("bad sort {}", _x))),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
/** EdMods is a set of edit changes to an [Calendar]. */
pub struct EdMods {
    #[serde(default)]
    info: FileInfo,
    /** the [HolydayMod]s in this EdMods */
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
        println!(
            "{}",
            Green.paint(format!("modifications read from reader with {:?}", u.info))
        );
        Ok(u)
    }
}
impl From<&mut Calendar> for EdMods {
    fn from(c: &mut Calendar) -> Self {
        Self {
            info: c.info.clone(),
            holydays: c
                .holydays
                .iter_mut()
                .map(|e| {
                    let e: Holyday = e.clone().into();
                    HolydayMod::from(e)
                })
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
        let title = self.clone().title.ok_or_else(|| {
            CalendarError::new(&format!(
                "adding holy day and field not specified - title, check that the edit tag ({:?}) is found in a calendar",
                self.tag
            ))
        })?;
        let e = Holyday {
            title: title.clone(),
            description: self.clone().description.unwrap_or(title.clone()),
            main: self.main.clone().ok_or_else(|| {
                CalendarError::new("adding holy day and field not specified -  main")
            })?,
            other: self.other.clone().ok_or_else(|| {
                CalendarError::new("adding holy day and field not specified -  other ")
            })?,
            death: self.death.clone().unwrap_or("".to_string()),
            refs: self.refs.clone().ok_or_else(|| {
                CalendarError::new("adding holy day and field not specified -  refs")
            })?,
            class: self
                .class
                //    .clone()
                .ok_or_else(|| {
                    CalendarError::new("adding holy day and field not specified -  class ")
                })?,
            tag: self.tag.clone(),
            has_eve: self.has_eve.ok_or_else(|| {
                CalendarError::new("adding holy day and field not specified -  has_eve")
            })?,
            date_cal: self
                .date_cal
                .clone() // igadding holy day and field not specified - re clippy
                .ok_or_else(|| {
                    CalendarError::new("adding holy day and field not specified -  date_cal ")
                })?,
            transfer: self.transfer.clone().ok_or_else(|| {
                CalendarError::new("adding holy day and field not specified -  transfer ")
            })?,
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
            title: some_unless_blank(&e.title),
            description: some_unless_blank(&e.description),
            main: Some(e.main),
            other: Some(e.other),
            death: some_unless_blank(&e.death),
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
fn some_unless_blank(s: &str) -> Option<String> {
    if s == "" {
        None
    } else {
        Some(s.to_string())
    }
}

/** Holy DayClass is the level of the holy day and can be commemoration, lesser
festival, festival, principal feast, also unclassified and (ordinary) Sunday*/
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
/** DateCal is an instruction to calculate a date e.g. 25 Dec, 2 days before
Easter Sunday. */
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
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
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Hash)]
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
        Self { wd }
    }
}
impl From<OrderableDayOfWeek> for chrono::Weekday {
    fn from(odw: OrderableDayOfWeek) -> Self {
        odw.wd
    }
}

/** TransferType indicates whether and how an holy day can be transferred
to another date or dropped */
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
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
    /** must occur before the specified date, otherwise drop */
    Before(DateCal),
    /** do not transfer */
    DoNotTransfer,
}
/** A principal attribute of a [Holyday] (at preseent only [MainAttribute::Martyr])   */
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
    pub fn url(&self) -> String {
        (self.website.prefix() + &self.article).clone()
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
/** the colour for a [Holyday] */
#[derive(Debug, Copy, Clone)]
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
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
/** list all [ProvHolyday]s, grouped by date, for reports */
#[derive(Default, Debug)]
pub struct ProvHolydaysByDate {
    phbd: HashMap<DateCal, Vec<ProvHolyday>>,
}
impl ProvHolydaysByDate {
    /** load all the [Holyday]s from a [Calendar] */
    pub fn load_calendar(&mut self, cal: &Calendar) {
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
    pub fn dates(&self) -> Keys<DateCal, Vec<ProvHolyday>> {
        self.phbd.keys()
    }
    /** get the   [ProvHolyday]s for a date */
    pub fn by_date(&self, date: DateCal) -> impl Iterator<Item = &ProvHolyday> {
        self.phbd.get(&date).unwrap().into_iter()
    }
}
/**  list all [ProvHolyday]s, grouped by tag, for reports */
#[derive(Default, Debug)]
pub struct ProvHolydaysByTag {
    phbt: HashMap<String, Vec<ProvHolyday>>,
}
impl ProvHolydaysByTag {
    /** load all the [Holyday]s from a [Calendar] */
    pub fn load_calendar(&mut self, cal: &Calendar) {
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
    /**  get all the tags with  [Holyday]s */
    pub fn tags(&self) -> Keys<String, Vec<ProvHolyday>> {
        self.phbt.keys()
    }
    /**  get the   [ProvHolyday]s for a tag */
    pub fn by_tag(&self, tag: String) -> impl Iterator<Item = &ProvHolyday> {
        self.phbt.get(&tag).unwrap().into_iter()
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
[licenses](https://www.gnu.org/licenses/). */
