/*! This the main program. It generates on ical calendar from an input file. See the Opts struct for the command line options.

This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/
extern crate structopt;
use anglican_calendar::calendar;
use anglican_calendar::year_calendar;
use ansi_term::Colour::*;
use std::fs::File;
use std::io::Write;
use std::io::{BufReader, BufWriter};
use structopt::StructOpt;

fn main() {
    println!("Copyright ©2019 Martin Ellison. This program comes with ABSOLUTELY NO WARRANTY. This is free software, and you are welcome to redistribute it under the GPL3 licence; see the README file for details.");
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
    if let Some(ical_fn) = opt.ical_filename {
        println!(
            "{}",
            Green.paint(format!("writing year calendar {}", ical_fn))
        );
        let of = File::create(ical_fn).map_err(calendar::CalendarError::from_error)?;
        let mut bw = BufWriter::new(of);
        bw.write(ical.to_string().as_bytes())
            .map_err(calendar::CalendarError::from_error)?;
        bw.flush().map_err(calendar::CalendarError::from_error)?;
    }

    if let Some(dfn) = opt.ical_del_filename {
        let of = File::create(dfn).map_err(calendar::CalendarError::from_error)?;
        let mut bw = BufWriter::new(of);
        bw.write(ical_del.to_string().as_bytes())
            .map_err(calendar::CalendarError::from_error)?;
        bw.flush().map_err(calendar::CalendarError::from_error)?;
    }
    if let Some(report_fn) = opt.report_filename {
        println!(
            "{}",
            Green.paint(format!("writing year calendar report {}", report_fn))
        );
        let of = File::create(report_fn).map_err(calendar::CalendarError::from_error)?;
        let mut bw = BufWriter::new(of);
        year_cal.write_report(&mut bw)?;
        bw.flush().map_err(calendar::CalendarError::from_error)?;
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
    /// unique identifier for calendar **do not use** domain name or email address
    #[structopt(short = "u", long = "unique")]
    unique: String,
}
