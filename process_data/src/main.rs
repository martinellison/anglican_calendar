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
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::Path;
use std::str::FromStr;
use structopt::StructOpt;

//mod tagfix;

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
    match opt.province {
        None => {
            get_cofe(&mut holydays_data, &opt.base_dir, opt.verbose);
            get_hkskh(&mut holydays_data, &opt.base_dir, opt.verbose);
            get_ecusa(&mut holydays_data, &opt.base_dir, opt.verbose);
            get_aca(&mut holydays_data, &opt.base_dir, opt.verbose);
        }
        Some(Province::ChurchOfEngland) => {
            get_cofe(&mut holydays_data, &opt.base_dir, opt.verbose);
        }
        Some(Province::ECUSA) => {
            get_ecusa(&mut holydays_data, &opt.base_dir, opt.verbose);
        }
        Some(Province::HongKong) => {
            get_hkskh(&mut holydays_data, &opt.base_dir, opt.verbose);
        }
        Some(Province::Australia) => {
            get_aca(&mut holydays_data, &opt.base_dir, opt.verbose);
        }
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
    /// List tags
    #[structopt(short = "l", long = "listtags")]
    tag_list: bool,
    /// Report by date
    #[structopt(short = "d", long = "date")]
    date_report: bool,
    /// Report by tag
    #[structopt(short = "t", long = "tag")]
    tag_report: bool,
    /// Output calendar data file    
    #[structopt(short = "o", long = "output")]
    out_file: Option<String>,
    /// Only one province
    #[structopt(short = "p", long = "province")]
    province: Option<Province>,
    /// Base directory for input
    #[structopt(short = "b", long = "base")]
    base_dir: String,
}

//const VERBOSE: bool = true;
/// whatever has a calendar
#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
enum Province {
    ChurchOfEngland,
    HongKong,
    ECUSA,
    Australia,
}
impl FromStr for Province {
    type Err = calendar::CalendarError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cofe" | "en" => Ok(Province::ChurchOfEngland),
            "hkskh" | "hk" => Ok(Province::HongKong),
            "ecusa" | "tec" | "usa" | "us" => Ok(Province::ECUSA),
            "aca" | "au" => Ok(Province::Australia),
            _ => Err(calendar::CalendarError::new(&format!(
                "unknown province {}",
                &s
            ))),
        }
    }
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
    province: Province,
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
impl Holyday {
    fn cmp_by_date(a: &Self, b: &Self) -> Ordering {
        let c = a.date.cmp(&b.date);
        if c == Ordering::Equal {
            a.tag.cmp(&b.tag)
        } else {
            c
        }
    }
    fn cmp_by_tag(a: &Self, b: &Self) -> Ordering {
        let c = a.tag.cmp(&b.tag);
        if c == Ordering::Equal {
            a.date.cmp(&b.date)
        } else {
            c
        }
    }
}
struct HolydaysData {
    holydays: Vec<Holyday>,
    tags: HashMap<String, HashSet<Province>>,
    tag_fixes: HashMap<String, String>,
    extra_tag_fixes: HashMap<String, String>,
    bp_re: Regex,
    the_re: Regex,
    //    dash_re: Regex,
    verbose: bool,
}
impl HolydaysData {
    fn new(base_dir: &str, verbose: bool) -> Result<Self, calendar::CalendarError> {
        let fix_fn = base_dir.to_owned() + "/fixes.ron";
        // let reader = File::open(&fix_fn).map_err(calendar::CalendarError::from_error)?;
        let tag_fixes: HashMap<String, String> = get_map(&fix_fn)?;
        let extra_fix_fn = base_dir.to_owned() + "/extra-fixes.ron";
        let extra_tag_fixes: HashMap<String, String> = get_map(&extra_fix_fn)?;
        // from_reader(reader).map_err(calendar::CalendarError::from_error)?;
        Ok(Self {
            holydays: vec![],
            tags: HashMap::new(),
            tag_fixes,
            extra_tag_fixes,
            bp_re: Regex::new(r"\s*\([^)]+\)\s*").unwrap(),
            the_re: Regex::new(r"^the\s+").unwrap(),
            //       dash_re: Regex::new(r"\s*[-].*").unwrap(),
            verbose,
        })
    }
    fn add(&mut self, holyday: &mut Holyday) {
        let extra_tag = format!("-{:?}-{}", holyday.province, holyday.date);
        holyday.tag = self.fix_tag(&holyday.tag, &extra_tag);
        // .tag_fixes
        // .get(&holyday.tag)
        // .unwrap_or(&holyday.tag)
        // .to_string();
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
    fn fix_tag(&self, tag: &str, dist: &str) -> String {
        let t1 = tag
            .replace("saint", "")
            .replace("the evangelist", "")
            .replace("the apostle", "")
            .replace("of our lord jesus christ", "")
            .replace("of our lord", "")
            .replace("of jesus", "")
            .replace("of christ", "")
            .replace(" day", " ")
            .replace("  ", " ")
            .trim()
            .to_string()
            .replace("[^A-Za-z ]+", " ")
            .to_lowercase();
        let t2: String = self.bp_re.replace(&t1, " ").to_owned().to_string();
        let t3: String = self.the_re.replace(&t2, " ").to_owned().to_string();
        //   let t3a: String = self.dash_re.replace(&t3, " ").to_owned().to_string();
        let t3a = match t3.find(" – ") {
            None => t3.as_str(),
            Some(pos) => &t3[..pos],
        };
        let t3b = match t3a.find(",") {
            None => t3a,
            Some(pos) => &t3a[..pos],
        };
        let t4 = self
            .tag_fixes
            .get(t3b)
            .unwrap_or(&t3b.to_string())
            .to_string();
        let t5 = self
            .extra_tag_fixes
            .get(&(t4.clone() + dist))
            .unwrap_or(&t4.clone())
            .trim()
            .to_string();
        if self.verbose {
            println!(
                "{}",
                Green.on(Blue).paint(format!(
                    "'{}' tidied '{}' > '{}' > '{}' > '{}' > '{}' > '{}'",
                    tag, t2, t3, t3a, t3b, t4, t5
                ))
            );
        }
        t5
    }
}
fn get_map(file_name: &str) -> Result<HashMap<String, String>, calendar::CalendarError> {
    let reader = File::open(file_name).map_err(calendar::CalendarError::from_error)?;
    from_reader(reader).map_err(calendar::CalendarError::from_error)
}

fn get_cofe(mut holydays_data: &mut HolydaysData, base_dir: &str, verbose: bool) {
    if let Some(cal_text) = get_input(&(base_dir.to_owned() + "/original/cofe.txt")) {
        process_input(
            &cal_text,
            Province::ChurchOfEngland,
            &mut holydays_data,
            verbose,
        );
    }
}
fn get_hkskh(mut holydays_data: &mut HolydaysData, base_dir: &str, verbose: bool) {
    if let Some(cal_text) = get_input(&(base_dir.to_owned() + "/original/hkskh.txt")) {
        process_input(&cal_text, Province::HongKong, &mut holydays_data, verbose);
    }
}
fn get_ecusa(mut holydays_data: &mut HolydaysData, base_dir: &str, verbose: bool) {
    if let Some(cal_text) = get_input(&(base_dir.to_owned() + "/original/ecusa.txt")) {
        process_input(&cal_text, Province::ECUSA, &mut holydays_data, verbose);
    }
}
fn get_aca(mut holydays_data: &mut HolydaysData, base_dir: &str, verbose: bool) {
    if let Some(cal_text) = get_input(&(base_dir.to_owned() + "/original/aca.txt")) {
        process_input(&cal_text, Province::Australia, &mut holydays_data, verbose);
    }
}

fn get_input(file_name: &str) -> Option<String> {
    println!(
        "{}",
        Red.on(White)
            .bold()
            .paint(format!("---getting input {} ---", file_name))
    );
    // Create a path to the desired file
    //  let path = Path::new("../data/original/hkskh.txt");
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
    province: Province,
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
    let fixed_re = Regex::new(r"^\*\s*([0-9]+)[:]?\s+('*)(.+)'*$").unwrap();
    //  let fixed_re = Regex::new(r"^\*([0-9]+)\s+('*)([^']+)'*$").unwrap();
    let moveable_re = Regex::new(r"^\*('*)\s*(.+)$").unwrap();
    // let split_re = match province {
    //     Province::HongKong => Regex::new(r"'*\s*[.]\s+").unwrap(),
    //     _ => Regex::new(r"'*\s*[,]\s+").unwrap(),
    // };
    let split_re = Regex::new(r"'*\s*[@]\s+").unwrap();
    let year_re = Regex::new(r"[0-9]").unwrap();
    let tidy_re = Regex::new(r"['\[\]]").unwrap();
    let tidy2_re = Regex::new(r" - .*$").unwrap();
    //   let tag_fixes = tagfix::tag_fixes();
    let mut current_month: Option<u8> = None;
    //  let divb = Yellow.paint("|");

    //    let split_re = Regex::new(r"\s*[.,]\s+").unwrap();
    for line in in_data.split('\n') {
        let mut ok = false;
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
                } else {
                    println!(
                        "{}",
                        Red.bold()
                            .paint(format!(" unknown header '{}'", cap1.as_str()))
                    );
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
                    print!("{} ({})", cap1.as_str(), day);
                }
                if let Some(cap2) = cap.get(2) {
                    line_fmt = cap2.as_str().to_string();
                    if verbose {
                        print!("/-{}-", line_fmt);
                    }
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
                        &holydays_data,
                        verbose,
                    );
                }
                for s in split {
                    if s != "" {
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
                            if verbose {
                                print!("{}{}", Yellow.bold().paint("|"), &s);
                            }
                            holyday
                                .other
                                .push(tidy(s, &tidy_re, &tidy2_re, &mut holyday.refs));
                        }
                    }
                }
                if verbose {
                    println!(")");
                }
                holydays_data.add(&mut holyday);
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
                    ok = true;
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
            if ok {
                holydays_data.add(&mut holyday);
            } else {
                println!(
                    "{}",
                    Red.bold()
                        .paint(format!("line not good - omitted: {}", line))
                );
            }
        // }
        } else if line != "" {
            println!("comment: {}", line);
        }
    }
}
fn head_for_holyday(
    head: &str,
    line_fmt: &str,
    holyday: &mut Holyday,
    tidy_re: &Regex,
    tidy2_re: &Regex,
    data: &HolydaysData,
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
    let verbose = opts.verbose;
    if opts.tag_list {
        println!("{}", Green.on(Purple).bold().paint("tag report"));
        let mut keys: Vec<&String> = holydays_data.tags.keys().collect();
        keys.sort();
        for k in keys {
            let n = holydays_data.tags.get(k).unwrap();
            println!("{} = {:?}", k, n);
        }
    }

    if opts.date_report {
        println!(
            "{}",
            Green.on(Purple).bold().paint("holyday report by date")
        );
        holydays_data
            .holydays
            .sort_by(|a, b| Holyday::cmp_by_date(a, b));
        let mut last_date: Option<HolydayDate> = None;
        for e in &holydays_data.holydays {
            if Some(e.date) != last_date {
                //  if verbose {
                println!("{}", Green.bold().paint(format!("{:?}", e.date)));
                //    }
                last_date = Some(e.date);
            }
            //  if verbose {
            println!(
                "{} {}: {} {}",
                Blue.bold().paint(format!("{}", e.tag)),
                Yellow.paint(format!("{:?}", e.province)),
                e.name,
                e.source
            );
            //  }
        }
    }
    if opts.tag_report {
        println!("{}", Green.on(Purple).bold().paint("holyday report by tag"));
        holydays_data
            .holydays
            .sort_by(|a, b| Holyday::cmp_by_tag(a, b));
        let mut last_tag: Option<String> = None;
        let mut prev: Option<Holyday> = None;
        for e in &holydays_data.holydays {
            let exp = Some(e.tag.clone());
            if exp != last_tag {
                if verbose {
                    println!("{}", Green.bold().paint(format!("{:}", e.tag)));
                }
                last_tag = exp;
                prev = None;
            }
            let problem = if let Some(p) = prev.clone() {
                if e.tag == "" {
                    println!("{}", Red.on(White).paint("no tag"));
                    true
                } else if p.date != e.date {
                    println!(
                        "{}",
                        Red.on(White)
                            .paint(format!("{} differ {}/{}", e.tag, p.date, e.date))
                    );
                    true
                } else {
                    false
                }
            } else {
                false
            };
            if problem && !verbose {
                if let Some(p) = prev.clone() {
                    println!(
                        " {}: {} {} {}",
                        Yellow.bold().paint(format!("{:?}", p.province)),
                        p.date,
                        p.name,
                        Cyan.paint(&p.source)
                    );
                }
            }
            if verbose || problem {
                println!(
                    " {}: {} {} {}",
                    Yellow.bold().paint(format!("{:?}", e.province)),
                    e.date,
                    e.name,
                    Cyan.paint(&e.source)
                );
            }
            prev = Some(e.clone());
        }
    }
    if let Some(of) = &opts.out_file {
        println!(
            "{}",
            Green.on(Purple).bold().paint("writing to calendar file")
        );
        let mut cal = calendar::Calendar::new();
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
                transfer: e.transfer,
            };
            if opts.verbose {
                println!("{:#?} / {:#?}", e, ce);
            }
            cal.add(&ce);
        }
        cal.write(&mut bw).unwrap();
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
