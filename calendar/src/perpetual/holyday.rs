/*! [Holyday] code */
/// a value or an error code
// type Result<T> = std::result::Result<T, CalendarError>;
use super::{DateCal, HolydayClass, HolydayRef, MainAttribute, TransferType};
use crate::perpetual::Reference;
use convert_case::{Case, Casing};
use databake::Bake;
use getset::{CopyGetters, Getters, MutGetters};
use itertools::Itertools;
use serde_derive::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::HashSet,
    error::Error,
    hash::{Hash, Hasher},
};

/** An Holy Day is an holy day in a [Calendar] e.g. the holy days of the Anglican
Church of Hong Kong include Easter Sunday and Matteo Ricci.*/
#[derive(
    Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Getters, MutGetters, CopyGetters, Bake,
)]
#[databake(path = calendar::calendar)]
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
    #[getset(get_copy = "pub(crate)")]
    pub class: HolydayClass,
    /** a short tag for identifying the holy day so that it can be
    overridden */
    pub tag: String,
    /** holy day has eve (and eve is not an specified holy day in its
    own right) */
    #[getset(get_copy = "pub(crate)")]
    pub has_eve: bool,
    /** date calculation */
    pub date_cal: DateCal,
    /** whether and how the holy day must be transferred to another
    date or dropped */
    #[getset(get_copy = "pub(crate)")]
    pub transfer: TransferType,
}
impl Hash for Holyday {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.title.hash(state);
        self.tag.hash(state);
        self.date_cal.hash(state);
    }
}
impl Holyday {
    // /** modify an Holy Day according to an HolydayMod */
    // pub fn modify(&mut self, m: &HolydayMod) {
    //     if let Some(t) = &m.title {
    //         self.title = t.to_string();
    //     }
    //     if let Some(mn) = &m.main {
    //         self.main.clone_from(mn);
    //     }
    //     if let Some(o) = &m.other {
    //         self.other.clone_from(o);
    //     }
    //     if let Some(d) = &m.death {
    //         self.death = d.to_string();
    //     }
    //     if let Some(rr) = &m.refs {
    //         self.refs.clone_from(rr);
    //     }
    //     if let Some(c) = &m.class {
    //         self.class = *c;
    //     }
    //     if let Some(e) = &m.has_eve {
    //         self.has_eve = *e;
    //     }
    //     if let Some(c) = &m.date_cal {
    //         self.date_cal = c.clone();
    //     }
    //     if let Some(t) = &m.transfer {
    //         self.transfer = *t;
    //     }
    // }

    pub(crate) fn cmp_by_date_cal(&self, other: &Self) -> Ordering {
        self.date_cal.cmp(&other.date_cal)
    }

    pub(crate) fn cmp_by_tag(&self, other: &Self) -> Ordering { self.tag.cmp(&other.tag) }

    /** `clean_up` tidies up a Holyday */
    pub fn clean_up(&mut self) {
        self.tag = self.tag.to_case(Case::Title);
        self.date_cal = self.date_cal.fixed();
        self.refs = self.refs.iter().unique().cloned().collect();
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
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
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
    fn from(rf: HolydayRef) -> Self { rf.r.as_ref().borrow().clone() }
}
