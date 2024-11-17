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
    path::{Path, PathBuf},
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

    // if let Some(dfn) = opt.ical_del_filename {
    //     let of = File::create(dfn).map_err(CalendarError::from_error)?;
    //     let mut bw = BufWriter::new(of);
    //     bw.write(ical_del.to_string().as_bytes())
    //         .map_err(CalendarError::from_error)?;
    //     bw.flush().map_err(CalendarError::from_error)?;
    // }
    match opt.cmd {
        Command::ICal {
            year,
            ical_filename,
            unique,
        } => {
            println!(
                "{}",
                Green.paint(format!("writing year calendar {ical_filename:?}",))
            );
            let year_cal = YearCalendar::from_calendar(&cal, year, opt.verbose)?;
            let ident = format!("{}-{}", unique, year);
            let (ical, _ical_del) = year_cal.to_ical(ident.as_str());
            let of = File::create(ical_filename).map_err(CalendarError::from_error)?;
            let mut bw = BufWriter::new(of);
            bw.write(ical.to_string().as_bytes())
                .map_err(CalendarError::from_error)?;
            bw.flush().map_err(CalendarError::from_error)?;
        },
        Command::Report {
            year,
            report_filename,
            wall_format,
        } => {
            println!(
                "{}",
                Green.paint(format!(
                    "writing year calendar report {:?}",
                    report_filename
                ))
            );
            let year_cal = YearCalendar::from_calendar(&cal, year, opt.verbose)?;
            let of = File::create(report_filename).map_err(CalendarError::from_error)?;
            let mut bw = BufWriter::new(of);
            if wall_format {
                year_cal.write_wall_calendar(&mut bw)
            } else {
                year_cal.write_report(&mut bw)
            }?;
            bw.flush().map_err(CalendarError::from_error)?;
        },
        Command::PerpetualReport {
            perpetual_report_filename,
        } => {
            println!(
                "{}",
                Green.paint(format!(
                    "writing perpetual calendar report {:?}",
                    perpetual_report_filename
                ))
            );
            let pof = File::create(perpetual_report_filename).map_err(CalendarError::from_error)?;
            let mut pbw = BufWriter::new(pof);
            cal.write_perpetual_report(&mut pbw)?;
            pbw.flush().map_err(CalendarError::from_error)?;
        },
    }
    Ok(())
}
#[derive(StructOpt, Debug)]
#[structopt(name = "anglican_calendar", about = "Process ecclesiastical calendars")]
/// Options from the command line
pub struct Opt {
    /// Print some debugging messages
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,
    /// Calendar file to use
    #[structopt(short = "c", long = "calendar")]
    calendar_filename: String,
    // /// iCal output file for deletion (apparently does not work)
    // #[structopt(short = "d", long = "delical")]
    // ical_del_filename: Option<String>,
    /// read from old format input file
    #[structopt(short = "f", long = "from_old_format")]
    from_old_format: bool,
    /// Command
    #[structopt(subcommand)]
    cmd: Command,
}
#[derive(StructOpt, Debug)]
#[structopt(about = "Command")]
enum Command {
    /// Create an iCalendar that can be loaded into Google Calendar and the like
    ICal {
        /// Year e.g. 2024
        #[structopt(short = "y", long = "year")]
        year: i32,
        /// iCal output file
        #[structopt(short = "i", long = "ical")]
        ical_filename: PathBuf,
        /// unique identifier for calendar **do not use domain name or email
        /// address**
        #[structopt(short = "u", long = "unique")]
        unique: String,
    },
    /// Create a report for a given year
    Report {
        /// Year e.g. 2024
        #[structopt(short = "y", long = "year")]
        year: i32,
        /// report output file (should be html)
        #[structopt(short = "r", long = "report")]
        report_filename: PathBuf,
        /// wall calendar format
        #[structopt(short = "w", long = "wall")]
        wall_format: bool,
    },
    /// Create a perpetual report (for all years)
    PerpetualReport {
        /// output file for a report on the perpetual calendar (should be html)
        //#[structopt(short = "p", long = "perpetual-report")]
        #[structopt(short = "r", long = "report")]
        perpetual_report_filename: PathBuf,
    },
}
