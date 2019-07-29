/*! Parses calendars from Wikipedia..

`process_data` reads calendar data extracted from Wikipedia in Wiki
markup format, and outputs a calendar data file. (The calendar data
will need to be cleaned up e.g. by using `edit_data`, before it can be
fed into the main program).

This code is very ad-hoc, and could do with a rewrite, but it is
intended to only be used once per calendar, so it is probably not
worth the trouble to rewrite it.
*/
extern crate anglican_calendar;
extern crate structopt;
use anglican_calendar::calendar;
use ansi_term::Colour::*;
use regex::Regex;
use ron::de::from_reader;
use ron::ser::to_string_pretty;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::Path;
use structopt::StructOpt;

fn main() {
    println!("Copyright ©2019 Martin Ellison. This program comes with ABSOLUTELY NO WARRANTY. This is free software, and you are welcome to redistribute it under the GPL3 licence; see the README file for details.");
    if let Err(e) = run() {
        println!("failed with error {:?}", e);
    }
}
fn run() -> Result<(), calendar::CalendarError> {
    let opt = Opt::from_args();
    if opt.verbose {
        println!("getting holydays data");
    }
    let mut holydays_data = HolydaysData::new(&opt.base_dir, opt.verbose)?;
    let prov_list = calendar::ProvinceList::make();
    match opt.province {
        None => {
            for (_p, pd) in prov_list.all() {
                get_province(&pd, &mut holydays_data, &opt.base_dir, opt.verbose)
            }
        }
        Some(calendar::Province::ChurchOfEngland)
        | Some(calendar::Province::ECUSA)
        | Some(calendar::Province::HongKong)
        | Some(calendar::Province::Australia)
        | Some(calendar::Province::SouthAfrica)
        | Some(calendar::Province::Canada)
        | Some(calendar::Province::BCP) => {
            let pd = prov_list.get(opt.province.unwrap());
            get_province(&pd, &mut holydays_data, &opt.base_dir, opt.verbose)
        }
        _ => panic!(format!("bad province {:?}", opt.province)),
    }
    if opt.verbose {
        println!("reporting");
    }
    report(&mut holydays_data, &opt);
    Ok(())
}
#[derive(StructOpt, Debug)]
#[structopt(name = "", about = "Reformat calendars from wikipedia")]
/// Options from the command line
pub struct Opt {
    /// Print some debugging messages
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,
    /// Output calendar data file    
    #[structopt(short = "o", long = "output")]
    out_file: Option<String>,
    /// Only one province
    #[structopt(short = "p", long = "province")]
    province: Option<calendar::Province>,
    /// Base directory for input
    #[structopt(short = "b", long = "base")]
    base_dir: String,
    /// Description for output file
    #[structopt(long = "descr")]
    descr: Option<String>,
}
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
enum HolydayDate {
    Fixed { month: u8, day: u8 },
    Moveable,
}
impl fmt::Display for HolydayDate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HolydayDate::Fixed { month, day } => write!(f, "{}-{}", month, day),
            HolydayDate::Moveable => write!(f, "M"),
        }
    }
}
#[derive(Clone, Debug)]
struct Holyday {
    date: HolydayDate,
    province: calendar::Province,
    name: String,
    tag: String,
    source: String,
    martyr: bool,
    death_date: String,
    other: Vec<String>,
    class: calendar::HolydayClass,
    refs: Vec<String>,
    transfer: calendar::TransferType,
}
impl Holyday {}
struct HolydaysData {
    base_dir: String,
    holydays: Vec<Holyday>,
    tags: HashMap<String, HashSet<calendar::Province>>,
    tag_fixes: HashMap<String, String>,
    extra_tag_fixes: HashMap<String, String>,
    bp_re: Regex,
    //   the_re: Regex,
    white_re: Regex,
    phrase_re: Regex,
    phrase_sp_re: Regex,
    dash_re: Regex,
    verbose: bool,
    as_fixed: HashMap<String, String>,
    as_unfixed: HashSet<String>,
}
impl HolydaysData {
    fn new(base_dir: &str, verbose: bool) -> Result<Self, calendar::CalendarError> {
        let fix_fn = base_dir.to_owned() + "/fixes.ron";
        let tag_fixes: HashMap<String, String> = get_map(&fix_fn)?;
        let extra_fix_fn = base_dir.to_owned() + "/extra-fixes.ron";
        let extra_tag_fixes: HashMap<String, String> = get_map(&extra_fix_fn)?;
        Ok(Self {
            base_dir: base_dir.to_string(),
            holydays: vec![],
            tags: HashMap::new(),
            tag_fixes,
            extra_tag_fixes,
            bp_re: Regex::new(r"\s*\([^)]+\)\s*").unwrap(),
            //    the_re: Regex::new(r"^the\s+").unwrap(),
            white_re: Regex::new(r"\s+").unwrap(),
            phrase_re: Regex::new(r"the evangelist|the apostle|of our lord jesus christ|of our lord|of jesus|of the lord|st\.\s*|\ssts\s|his companions|her companions|and their companion|apostles|apostle|evangelist|deaconess|deacon|missionaries|missionary|bishop|abp|martyrs?|abbot|priest|monk|is celebrated.*|may be celebrated .*|may be kept.*|alternative .*|traditionally .*|also in.*|this festival").unwrap(),
            phrase_sp_re: Regex::new(r"saint | st |^st | day |[^A-Za-z ]+|^the\s+|\s+day$|which .*$|is .^$|of christ |of christ$|\([^)]+\)|\s+and\s+|^sts\s+").unwrap(),
            dash_re: Regex::new(r"\s*[-–].*$").unwrap(),
            verbose,
            as_fixed: HashMap::new(),
            as_unfixed: HashSet::new(),
        })
    }
    fn add(&mut self, holyday: &mut Holyday) {
        let extra_tag = format!("-{:?}-{}", holyday.province, holyday.date);
        holyday.tag = self.fix_tag(&holyday.tag, &extra_tag);
        match self.tags.get_mut(&holyday.tag) {
            Some(s) => {
                s.insert(holyday.province);
            }
            None => {
                let mut s = HashSet::new();
                s.insert(holyday.province);
                self.tags.insert(holyday.tag.clone(), s);
            }
        }
        self.holydays.push(holyday.clone());
    }
    fn fix_tag(&mut self, tag: &str, dist: &str) -> String {
        let tag1 = self.fix_tag_algo(tag);
        let (tag2, fixed) = self.fix_tag_fixes(&tag1, dist);
        if fixed {
            self.as_fixed.insert(tag1, tag2.clone());
        } else {
            self.as_unfixed.insert(tag1);
        }
        tag2
    }
    fn fix_tag_algo(&self, tag: &str) -> String {
        // let t1 = tag
        //     .to_lowercase()
        //     .replace("saint ", " ")
        //     .replace(" day", " ")
        //     .replace("[^A-Za-z ]+", " ");
        let t0 = self
            .dash_re
            .replace_all(&tag.to_lowercase(), " ")
            .to_owned()
            .to_string();
        let t1: String = self
            .phrase_sp_re
            .replace_all(&t0, " ")
            .to_owned()
            .to_string();
        let t1a = self.phrase_re.replace_all(&t1, "");
        let t2: String = self.bp_re.replace(&t1a, " ").to_owned().to_string();
        //  let t3: String = self.the_re.replace(&t2, " ").to_owned().to_string();
        let t4 = match t2.find(" – ") {
            None => t2.as_str(),
            Some(pos) => &t2[..pos],
        };
        let t5 = match t4.find(",") {
            None => t4,
            Some(pos) => &t4[..pos],
        };
        let t6 = self.white_re.replace_all(t5, " ");
        let t7 = t6.trim();
        if self.verbose {
            println!(
                "{}",
                Green.on(Blue).paint(format!(
                    "'{}' tidied > 0:'{}'  > 1:'{}'> 1a:'{}' > 2:'{}' > 4:'{}' > 5:'{}' > 6:'{}' > 7:'{}'",
                    tag,t0, t1, t1a, t2, t4, t5, t6, t7
                ))
            );
        }
        t7.to_string()
    }
    fn fix_tag_fixes(&self, tag: &str, dist: &str) -> (String, bool) {
        let mut fixed = false;
        if self.verbose {
            print!("{}", Green.on(Blue).paint(format!("'{}' replaced", tag)));
        }
        let tr2o = self.tag_fixes.get(tag);
        let tr2 = if let Some(tr2x) = tr2o {
            fixed = true;
            if self.verbose {
                print!("{}", Green.on(Blue).paint(format!("  > '{}' ", tr2x)));
            }
            tr2x
        } else {
            tag
        }
        .to_string();
        let tr3 = self
            .extra_tag_fixes
            .get(&(tr2.clone() + dist))
            .unwrap_or(&tr2.clone())
            .trim()
            .to_string();
        if self.verbose {
            println!("{}", Green.on(Blue).paint(format!("   > '{}'", tr3)));
        }
        (tr3, fixed)
    }
    fn dump_used_fixed(&self) {
        let ofx = self.base_dir.clone() + "/fixes-used.ron";
        println!(
            "{}",
            Blue.on(Green)
                .bold()
                .paint(format!("writing tag use to {}", ofx))
        );
        let f = File::create(ofx).unwrap();
        let mut bw = BufWriter::new(f);
        let s = to_string_pretty(&self.as_fixed, ron::ser::PrettyConfig::default()).unwrap();
        let _ = bw.write(s.as_bytes()).unwrap();
        let s = to_string_pretty(&self.as_unfixed, ron::ser::PrettyConfig::default()).unwrap();
        let _ = bw.write(s.as_bytes()).unwrap();
        bw.flush().unwrap()
    }
}
fn get_map(file_name: &str) -> Result<HashMap<String, String>, calendar::CalendarError> {
    let reader = File::open(file_name).map_err(calendar::CalendarError::from_error)?;
    from_reader(reader).map_err(calendar::CalendarError::from_error)
}
fn get_province(
    pd: &calendar::ProvinceData,
    mut holydays_data: &mut HolydaysData,
    base_dir: &str,
    verbose: bool,
) {
    if let Some(cal_text) = get_input(&format!("{}/original/{}.txt", base_dir, pd.abbrev)) {
        process_input(&cal_text, pd.province, &mut holydays_data, verbose);
    }
}

fn get_input(file_name: &str) -> Option<String> {
    println!(
        "{}",
        Red.on(White)
            .bold()
            .paint(format!("---getting input {} ---", file_name))
    );
    let path = Path::new(file_name);
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(&path) {
        // The `description` method of `io::Error` returns a string that
        // describes the error
        Err(why) => {
            println!(
                "{}",
                Red.on(White)
                    .bold()
                    .paint(format!("couldn't open {}: {:?}", display, why))
            );
            return None;
        }
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {:?}", display, why),
        Ok(_) => Some(s),
    }
}
fn process_input(
    in_data: &str,
    province: calendar::Province,
    holydays_data: &mut HolydaysData,
    verbose: bool,
) {
    if verbose {
        println!(
            "{}",
            Red.on(White)
                .bold()
                .paint(format!("processing input {:?}", province))
        )
    }
    let head_re = Regex::new(r"^=+\s*([a-zA-Z ]+)\s*=+$").unwrap();
    let fixed_re = match province {
        calendar::Province::Canada => Regex::new(r"^\s*([0-9]+)[@]()(.+)$").unwrap(),
        _ => Regex::new(r"^\*\s*([0-9]+)[:]?\s+('*)(.+)'*$").unwrap(),
    };
    let moveable_re = Regex::new(r"^\*('*)\s*(.+)$").unwrap();
    let split_re = match province {
        calendar::Province::Canada => Regex::new(r"\s*[@]\s*").unwrap(),
        _ => Regex::new(r"'*\s*[@]\s*").unwrap(),
    };
    let year_re = Regex::new(r"[0-9]").unwrap();
    let tidy_re = Regex::new(r"['\[\]]").unwrap();
    let tidy2_re = Regex::new(r" - .*$").unwrap();
    let mut current_month: Option<u8> = None;
    let mut last_class: Option<calendar::HolydayClass> = None;
    for line in in_data.split('\n') {
        //   let mut ok = false;
        if let Some(cap) = head_re.captures(line) {
            if verbose {
                print!("head");
            }
            if let Some(cap1) = cap.get(1) {
                let m_str = cap1.as_str().trim();
                if let Some(month) = string_to_month(&m_str) {
                    if verbose {
                        print!(
                            "{}",
                            Yellow.paint(format!(" month {} ({})", &month, &m_str))
                        );
                    }
                    current_month = Some(month);
                    last_class = None;
                } else {
                    match m_str {
                        "Principal Feasts" | "Fasts" => {
                            last_class = Some(calendar::HolydayClass::Principal)
                        }
                        "Feasts of Our Lord" => last_class = Some(calendar::HolydayClass::Festival),

                        _ => println!(
                            "{}",
                            Red.bold()
                                .paint(format!(" unknown header '{}'", cap1.as_str()))
                        ),
                    }
                }
            }
            if verbose {
                println!();
            }
        } else if let Some(cap) = fixed_re.captures(line) {
            let mut day = 0u8;
            if verbose {
                print!("fixed (");
            }
            let mut line_fmt: String = "".to_string();
            if let Some(cap1) = cap.get(1) {
                day = cap1.as_str().parse::<u8>().unwrap();
                if verbose {
                    print!("({})", day);
                }
                if let Some(cap2) = cap.get(2) {
                    line_fmt = cap2.as_str().to_string();
                    if verbose {
                        print!("/-{}-", line_fmt);
                    }
                } else {
                    println!("{}", Red.bold().paint(format!("no first {}", line)));
                }
            }
            if let Some(cap3) = cap.get(3) {
                let mut holyday = Holyday {
                    date: HolydayDate::Fixed {
                        month: current_month.unwrap_or_else(|| panic!("no month for {}", line)),
                        day,
                    },
                    province: province,
                    name: "".to_string(),
                    tag: "".to_string(),
                    source: line.to_string(),
                    martyr: false,
                    death_date: "".to_string(),
                    other: vec![],
                    class: calendar::HolydayClass::Commemoration,
                    refs: vec![],
                    transfer: calendar::TransferType::Normal,
                };
                let cap3a = cap3
                    .as_str()
                    .replace("<small>", "")
                    .replace("</small>", "")
                    .to_owned()
                    + "  ";
                //    print!("={}=", cap3a);
                let mut split = split_re.split(&cap3a);
                if let Some(head) = split.next() {
                    head_for_holyday(
                        head,
                        &line_fmt,
                        &mut holyday,
                        &tidy_re,
                        &tidy2_re,
                        holydays_data,
                        verbose,
                    );
                }
                for s in split {
                    let mut keep = true;
                    if s == "" {
                        continue;
                    }
                    if s.to_ascii_lowercase().contains("martyr") {
                        holyday.martyr = true;
                        if verbose {
                            print!("{}", Red.bold().paint("(Martyr)"));
                        }
                    }
                    if year_re.is_match(s) {
                        if verbose {
                            print!("{}date: {}", Yellow.bold().paint("|"), &s);
                        }
                        holyday.death_date = tidy(s, &tidy_re, &tidy2_re, &mut holyday.refs);
                    } else {
                    }
                    if province == calendar::Province::Canada {
                        let corr = s.to_ascii_lowercase().trim().to_string();
                        match corr.as_str() {
                            // TODO check Corpus Christi
                            "pf" => {
                                holyday.class = calendar::HolydayClass::Principal;
                                keep = false
                            }
                            "hd" => {
                                holyday.class = calendar::HolydayClass::Festival;
                                keep = false
                            }
                            "mem" => {
                                holyday.class = calendar::HolydayClass::LesserFestival;
                                keep = false;
                            }
                            "com" => {
                                holyday.class = calendar::HolydayClass::Commemoration;
                                keep = false;
                            }
                            _ => {
                                // println!(
                                //     "{}",
                                //     Yellow.bold().paint(format!(
                                //         "canada not class '{}'->'{}'",
                                //         s, &corr
                                //     ))
                                // );
                            }
                        }
                    }
                    if keep {
                        if verbose {
                            print!("{}{}", Yellow.bold().paint("|"), &s);
                        }
                        holyday
                            .other
                            .push(tidy(s, &tidy_re, &tidy2_re, &mut holyday.refs));
                    }
                }
                if verbose {
                    println!(")");
                }
                holydays_data.add(&mut holyday);
            } else {
                println!("{}", Red.bold().paint(format!("no third {}", line)));
            }
        } else if let Some(cap) = moveable_re.captures(line) {
            let mut holyday = Holyday {
                date: HolydayDate::Moveable,
                province: province,
                name: "".to_string(),
                tag: "".to_string(),
                source: line.to_string(),
                martyr: false,
                death_date: "".to_string(),
                other: vec![],
                class: calendar::HolydayClass::Commemoration,
                refs: vec![],
                transfer: calendar::TransferType::Normal,
            };
            let mut line_fmt: String = "".to_string();
            if verbose {
                print!("moveable");
            }
            if let Some(cap1) = cap.get(1) {
                if verbose {
                    line_fmt = cap1.as_str().to_string();
                    print!("/-{}-/", cap1.as_str());
                }
            }
            if let Some(cap_head) = cap.get(2) {
                let cap_heada = cap_head.as_str().to_owned() + "  ";
                let mut split = split_re.split(&cap_heada);
                if let Some(head) = split.next() {
                    head_for_holyday(
                        head,
                        &line_fmt,
                        &mut holyday,
                        &tidy_re,
                        &tidy2_re,
                        holydays_data,
                        verbose,
                    );
                }
                for s in split {
                    //           ok = true;
                    //      if s != "" {
                    holyday
                        .other
                        .push(tidy(s, &tidy_re, &tidy2_re, &mut holyday.refs));
                    if verbose {
                        print!("{}{}", Yellow.bold().paint("|"), &s);
                    }
                }
            }
            if verbose {
                println!();
            }
            if let Some(c) = last_class {
                holyday.class = c;
            }
            //    if ok {
            holydays_data.add(&mut holyday);
        // } else {
        //     println!(
        //         "{}",
        //         Red.bold()
        //             .paint(format!("line not good - omitted: {}", line))
        //     );
        // }
        // }
        } else if line != "" {
            println!("comment: {}", Yellow.bold().paint(line));
        }
    }
}
fn head_for_holyday(
    head: &str,
    line_fmt: &str,
    holyday: &mut Holyday,
    tidy_re: &Regex,
    tidy2_re: &Regex,
    data: &mut HolydaysData,
    verbose: bool,
) {
    holyday.class = fmt_to_class(&head, &line_fmt);
    holyday.name = tidy(head, &tidy_re, &tidy2_re, &mut holyday.refs);
    let extra_tag = format!("-{:?}-{}", holyday.province, holyday.date);
    holyday.tag = data.fix_tag(&holyday.name, &extra_tag);
    if verbose {
        print!(
            "{}{}->{:?}{}{}{}{}{}{}",
            Yellow.bold().paint("/"),
            head,
            holyday.class,
            Yellow.bold().paint("/"),
            holyday.name,
            Yellow.bold().paint("/"),
            holyday.tag,
            Yellow.bold().paint("+"),
            &extra_tag
        );
    }
}
fn fmt_to_class(head: &str, line_fmt: &str) -> calendar::HolydayClass {
    if line_fmt == "'''" {
        if head == head.to_ascii_uppercase() {
            //??
            calendar::HolydayClass::Principal
        } else {
            calendar::HolydayClass::Festival
        }
    } else if line_fmt == "''" {
        calendar::HolydayClass::LesserFestival
    } else if head.contains("</small>") {
        calendar::HolydayClass::Unclassified
    } else {
        calendar::HolydayClass::Commemoration
    }
}
fn tidy(head: &str, re: &Regex, re2: &Regex, refs: &mut Vec<String>) -> String {
    let mut buf = String::new();
    let mut result = String::new();
    for c in head.chars() {
        match c {
            '[' | ']' => {
                result.push_str(&buf);
                buf.clear();
            }
            '|' => {
                refs.push(buf.clone());
                buf.clear();
            }
            _ => buf.push(c),
        }
    }
    result.push_str(&buf);
    let res2 = re2.replace_all(&result, "");
    let res3 = re.replace_all(&res2, "").trim().to_string();
    println!(
        "{}",
        Black.on(White).paint(format!(
            "'{}' tidied '{}' > '{}' > '{}'",
            head, result, res2, res3
        ))
    );
    res3
}

fn string_to_month(s: &str) -> Option<u8> {
    match s.to_lowercase().as_str() {
        "january" => Some(1),
        "february" => Some(2),
        "march" => Some(3),
        "april" => Some(4),
        "may" => Some(5),
        "june" => Some(6),
        "july" => Some(7),
        "august" => Some(8),
        "september" => Some(9),
        "october" => Some(10),
        "november" => Some(11),
        "december" => Some(12),
        _ => None,
    }
}
fn report(holydays_data: &mut HolydaysData, opts: &Opt) {
    if let Some(of) = &opts.out_file {
        let mut cal = calendar::Calendar::new();
        let descr = opts.descr.clone().unwrap_or("".to_string());
        cal.info = calendar::FileInfo::new(&descr, "process data");
        if let Some(p) = opts.province {
            cal.province = p;
        }
        println!(
            "{}",
            Green.on(Purple).bold().paint(format!(
                "writing to calendar data file, province {:?}, info {:?}",
                cal.province, cal.info
            ))
        );
        let f = File::create(of).unwrap();
        let mut bw = BufWriter::new(f);
        for e in &holydays_data.holydays {
            let mut main: HashSet<calendar::MainAttribute> = HashSet::new();
            if e.martyr {
                main.insert(calendar::MainAttribute::Martyr);
            }
            let ce = calendar::Holyday {
                title: e.name.clone(),
                description: e.other.join(", "),
                main: main,
                other: e.other.clone(),
                death: e.death_date.clone(),
                refs: e
                    .refs
                    .clone()
                    .into_iter()
                    .map(|s| calendar::Reference::new(calendar::WebSite::Wikipedia, s))
                    .collect(),
                tag: e.tag.clone(),
                has_eve: false,
                date_cal: match e.date {
                    HolydayDate::Fixed { month, day } => calendar::DateCal::Fixed {
                        month: month,
                        day: day,
                    },
                    HolydayDate::Moveable => calendar::DateCal::Easter,
                },
                class: e.class,
                transfer: e.transfer.clone(),
            };
            if opts.verbose {
                println!("{:#?} / {:#?}", e, ce);
            }
            cal.add(&ce);
        }
        cal.write(&mut bw).unwrap();
    }
    holydays_data.dump_used_fixed();
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
