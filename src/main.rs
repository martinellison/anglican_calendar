/*! main program */
extern crate structopt;
use anglican_calendar::calendar;
use anglican_calendar::year_calendar;
use ansi_term::Colour::*;
//use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::io::{BufReader, BufWriter};
use structopt::StructOpt;

fn main() {
    color_backtrace::install();
    if let Err(e) = run() {
        println!("failed because {:#?}", e);
        panic!("failed");
    }
    println!("{}", Green.paint("done"));
}
fn run() -> Result<(), calendar::CalendarError> {
    let opt = Opt::from_args();
    println!(
        "{}",
        Green.paint(format!("reading calendar {}", opt.calendar_filename))
    );
    let inf = File::open(opt.calendar_filename).map_err(calendar::CalendarError::from_error)?;
    let mut br = BufReader::new(inf);
    let cal = calendar::Calendar::read(&mut br)?;
    let year_cal = year_calendar::YearCalendar::from_calendar(&cal, opt.year, opt.verbose)?;
    if opt.verbose {
        println!("{}", Green.paint("year calendar"));
        println!("{:#?}", year_cal);
    }
    println!("{}", Green.paint("generating year calendar"));
    let ident = format!(
        "{}-{}",
        opt.unique,
        //    chrono::Utc::now().format("%+"),
        opt.year
    );
    let (ical, ical_del) = year_cal.to_ical(ident.as_str());
    println!(
        "{}",
        Green.paint(format!("writing year calendar {}", opt.ical_filename))
    );
    let of = File::create(opt.ical_filename).map_err(calendar::CalendarError::from_error)?;
    let mut bw = BufWriter::new(of);
    bw.write(ical.to_string().as_bytes())
        .map_err(calendar::CalendarError::from_error)?;
    bw.flush().map_err(calendar::CalendarError::from_error)?;

    let of = File::create(opt.ical_del_filename).map_err(calendar::CalendarError::from_error)?;
    let mut bw = BufWriter::new(of);
    bw.write(ical_del.to_string().as_bytes())
        .map_err(calendar::CalendarError::from_error)?;
    bw.flush().map_err(calendar::CalendarError::from_error)?;
    Ok(())
}
#[derive(StructOpt, Debug)]
#[structopt(
    name = "anglican_calendar",
    about = "Process ecclestiastical calendars"
)]
/// Options from the command line
pub struct Opt {
    /// Print some debugging messages
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,
    /// Year
    #[structopt(short = "y", long = "year")]
    year: i32,
    /// Calendar file to use
    #[structopt(short = "c", long = "calendar")]
    calendar_filename: String,
    /// iCal output file
    #[structopt(short = "i", long = "ical")]
    ical_filename: String,
    /// iCal output file for deletion
    #[structopt(short = "d", long = "delical")]
    ical_del_filename: String,
    /// unique identifier for calendar e.g. dpmain name or email address
    #[structopt(short = "u", long = "unique")]
    unique: String,
}
