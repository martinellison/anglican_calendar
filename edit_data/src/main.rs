/*! This program can be used to edit the input data.

There are several kinds of edits that are possible.

* make changes to a calendar
* make the same changes to several calendars
* merge several calendars
* clean up a calendar file by reformatting it ('pretty')
* clean up an edit file by reformatting it ('pretty')

The merges and edits work using the tag field to identify each holy day.

Ther are two file formats:

* calendars file (also used by other programs in this group)
* edit files

Check the command line options for the specific details of how to carry out these edits. */
extern crate anglican_calendar;
extern crate structopt;
//use crate::calendar;
use anglican_calendar::calendar;
use ansi_term::Colour::*;
use ron::ser::to_string_pretty;
//use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::io::{BufReader, BufWriter};
use structopt::StructOpt;

fn main() {
    println!("Copyright ©2019 Martin Ellison. This program comes with ABSOLUTELY NO WARRANTY. This is free software, and you are welcome to redistribute it under the GPL3 licence; see the README file for details.");
    if let Err(e) = run() {
        println!("failed because {:?}", e);
        panic!("failed");
    }
    println!("{}", Green.paint("done"));
}
fn run() -> Result<(), calendar::CalendarError> {
    // println!("getting opts...");
    let opt = Opt::from_args();
    //  println!("got opts {:?}", &opt);
    let mut cal: Option<calendar::Calendar> = None;
    let is_editing = opt.in_file.is_some() && opt.out_file.is_some();
    if opt.in_file.is_some() {
        println!(
            "{}",
            Green.paint(format!("reading calendar {}", opt.in_file.clone().unwrap()))
        );
        let infn = opt.in_file.unwrap();
        let inf = File::open(&infn).map_err(calendar::CalendarError::from_error)?;
        let mut br = BufReader::new(inf);
        let mut read_cal = calendar::Calendar::read(&mut br)?;
        println!("calendar read");
        // write calendar as pretty if required
        if let Some(p) = &opt.pretty {
            println!("pretty printing");
            let pifn = format!("{}.{}", &infn, &p);
            let mut bwb = open_out_file(&pifn)?;
            let mut bw = bwb.as_mut();
            read_cal.write(&mut bw).unwrap()
        }
        // convert to edits if required
        if let Some(a) = &opt.as_edits {
            println!("converting to edits");
            let aifn = format!("{}.{}", &infn, &a);
            let mut bwb = open_out_file(&aifn)?;
            let bw = bwb.as_mut();
            let eds = calendar::EdMods::from(&mut read_cal);
            let s = to_string_pretty(&eds, ron::ser::PrettyConfig::default()).unwrap();
            let _ = bw.write(s.as_bytes()).unwrap();
            bw.flush().unwrap()
        }
        cal = Some(read_cal);
    }

    for ef in opt.edit_files {
        // apply edits
        println!("{}", Green.paint(format!("reading edits {}", &ef)));
        let edf = File::open(&ef).map_err(calendar::CalendarError::from_error)?;
        let mut ebr = BufReader::new(edf);
        println!("{}", Green.paint("interpreting edits"));
        let eds = calendar::EdMods::read(&mut ebr)?;
        println!("{}", Green.paint("applying edits"));
        if let Some(c) = &mut cal {
            c.apply(&eds)?;
        }
        // write edits as pretty if required
        if let Some(p) = &opt.pretty {
            println!("pretty printing");
            let pefn = format!("{}.{}", &ef, &p);
            let s = to_string_pretty(&eds, ron::ser::PrettyConfig::default()).unwrap();
            let mut bwb = open_out_file(&pefn)?;
            let bw = bwb.as_mut();
            let _ = bw.write(s.as_bytes()).unwrap();
            bw.flush().unwrap()
        }
    }
    match opt.sort {
        calendar::HolydaySort::NoSort => {}
        calendar::HolydaySort::Normal => {
            println!("sorting normally");
            if let Some(c) = &mut cal {
                c.sort();
            }
        }
        calendar::HolydaySort::DateCal => {
            println!("sorting by date");
            if let Some(c) = &mut cal {
                c.sort_by_date_cal();
            }
        }
        calendar::HolydaySort::Tag => {
            println!("sorting by yag");
            if let Some(c) = &mut cal {
                c.sort_by_tag();
            }
        }
    }

    if is_editing {
        println!(
            "{}",
            Green.paint(format!(
                "writing as edited {}",
                &opt.out_file.clone().unwrap()
            ))
        );
        // write calendar
        let mut bwb = open_out_file(opt.out_file.unwrap().as_str())?;
        let mut bw = bwb.as_mut();
        if let Some(mut c) = cal {
            let descr = opt.descr.clone().unwrap_or("".to_string());
            c.info = calendar::FileInfo::new(&descr, "edit data");
            c.write(&mut bw).unwrap()
        }
    }
    println!("done");
    Ok(())
}
fn open_out_file(fpath: &str) -> Result<Box<dyn Write>, calendar::CalendarError> {
    let of = File::create(fpath).map_err(calendar::CalendarError::from_error)?;
    Ok(Box::new(BufWriter::new(of)))
}

#[derive(StructOpt, Debug)]
#[structopt(name = "", about = "Edit calendars")]
/// Options from the command line
pub struct Opt {
    /// Input calendar data file
    #[structopt(short = "i", long = "input")]
    in_file: Option<String>,
    /// Input calendar edit file
    #[structopt(short = "e", long = "edit")]
    edit_files: Vec<String>,
    /// Output calendar data file
    #[structopt(short = "o", long = "output")]
    out_file: Option<String>,
    /// Pretty suffix -- used to pretty-print inputs
    #[structopt(short = "p", long = "pretty")]
    pretty: Option<String>,
    /// As-edits suffix -- used to convert calendar to edits
    #[structopt(short = "a", long = "asedits")]
    as_edits: Option<String>,
    /// Description for output file
    #[structopt(short = "d", long = "descr")]
    descr: Option<String>,
    /// Sort calendar data for output Normal/DateCal
    #[structopt(short = "s", long = "sort", default_value)]
    sort: calendar::HolydaySort,
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
