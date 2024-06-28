/*! This program is part of Anglican Calendar.

 This the main program. It generates an ical calendar from an input file. See the Opts struct for the command line options.

This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see
    [licenses](https://www.gnu.org/licenses/)..
*/
mod perpetual;
mod year_calendar;
use crate::{perpetual::CalendarError, year_calendar::year_calendar::YearCalendar};
use ansi_term::Colour::*;
use log::debug;
use simplelog::{LevelFilter, SimpleLogger};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::Path,
};
use structopt::StructOpt;
/// a value or an error code
type Result<T> = std::result::Result<T, CalendarError>;

fn main() {
    println!(
        "Copyright ©2019-2024 Martin Ellison. This program comes with ABSOLUTELY NO WARRANTY. \
         This is free software, and you are welcome to redistribute it under the GPL3 licence; \
         see the README file for details."
    );
    color_backtrace::install();
    if let Err(e) = run() {
        println!("failed because {:#?}", e);
        panic!("failed");
    }
    println!("{}", Green.paint("done"));
}
fn run() -> Result<()> {
    let opt = Opt::from_args();
    if opt.verbose {
        SimpleLogger::init(
            LevelFilter::Debug,
            simplelog::ConfigBuilder::default()
                .set_time_level(LevelFilter::Trace)
                .set_thread_level(LevelFilter::Trace)
                .set_location_level(LevelFilter::Error)
                .build(),
        )
        .map_err(|err| CalendarError::new(&format!("error when starting logger: {err}")))?;
        debug!("options: {:?}", opt);
    }
    println!(
        "{}",
        Green.paint(format!("reading calendar {}", opt.calendar_filename))
    );

    let cal = if opt.from_old_format {
        debug!("reading from old format file");
        let inf = File::open(opt.calendar_filename).map_err(CalendarError::from_error)?;
        let mut br = BufReader::new(inf);
        perpetual::calendar::Calendar::read(&mut br)?
    } else {
        perpetual::from_spreadsheet::read_from_spreadsheet(Path::new(&opt.calendar_filename))?
    };
    let year_cal = YearCalendar::from_calendar(&cal, opt.year, opt.verbose)?;
    // if opt.verbose {
    debug!("{}", Green.paint("year calendar"));
    // println!("{:#?}", year_cal);
    // }
    println!("{}", Green.paint("generating year calendar"));
    let ident = format!("{}-{}", opt.unique, opt.year);
    let (ical, ical_del) = year_cal.to_ical(ident.as_str());
    if let Some(ical_fn) = opt.ical_filename {
        println!(
            "{}",
            Green.paint(format!("writing year calendar {ical_fn}",))
        );
        let of = File::create(ical_fn).map_err(CalendarError::from_error)?;
        let mut bw = BufWriter::new(of);
        bw.write(ical.to_string().as_bytes())
            .map_err(CalendarError::from_error)?;
        bw.flush().map_err(CalendarError::from_error)?;
    }

    if let Some(dfn) = opt.ical_del_filename {
        let of = File::create(dfn).map_err(CalendarError::from_error)?;
        let mut bw = BufWriter::new(of);
        bw.write(ical_del.to_string().as_bytes())
            .map_err(CalendarError::from_error)?;
        bw.flush().map_err(CalendarError::from_error)?;
    }
    if let Some(report_fn) = opt.report_filename {
        println!(
            "{}",
            Green.paint(format!("writing year calendar report {}", report_fn))
        );
        let of = File::create(report_fn).map_err(CalendarError::from_error)?;
        let mut bw = BufWriter::new(of);
        year_cal.write_report(&mut bw)?;
        bw.flush().map_err(CalendarError::from_error)?;
    }
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
    ical_filename: Option<String>,
    /// report output file
    #[structopt(short = "r", long = "report")]
    report_filename: Option<String>,
    /// iCal output file for deletion (apparently does not work)
    #[structopt(short = "d", long = "delical")]
    ical_del_filename: Option<String>,
    /// unique identifier for calendar **do not use domain name or email
    /// address**
    #[structopt(short = "u", long = "unique")]
    unique: String,
    /// read from old format filea
    #[structopt(short = "f", long = "from_old_format")]
    from_old_format: bool,
}
