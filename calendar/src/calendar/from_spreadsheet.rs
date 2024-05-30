/*! inputs a [Calendar] from a spreadsheet */

use super::{Calendar, CalendarError, DateCal, Holyday};
use crate::calendar::{MainAttribute, Reference, WebSite::Wikipedia};
use calamine::{open_workbook, Data, DataType, Reader, Xlsx};
use log::debug;
use std::{path::Path, str::FromStr};

/** `read_from_spreadsheet` reads a [Calendar] from a spreadsheet */
pub fn read_from_spreadsheet(file: &Path) -> Result<Calendar, CalendarError> {
    let mut ss_err = None;
    let mut calendar = Calendar::new();
    let mut workbook: Xlsx<_> = open_workbook(file)?;
    let range = workbook.worksheet_range("Calendar")?;
    // note: values must be in the correct cell or they will fail
    let get_meta = |key: &str, row| {
        if let Some(key_cell) = range.get((row, 0)) {
            if key_cell.as_string() == Some(key.to_string()) {
                if let Some(data_cell) = range.get((row, 1)) {
                    Ok(data_cell.as_string().unwrap_or_default().trim().to_string())
                } else {
                    Err(CalendarError::new("Cannot find {key} value}"))
                }
            } else {
                Err(CalendarError::new("Cannot find {key} header}"))
            }
        } else {
            Err(CalendarError::new("Cannot find {key} header}"))
        }
    };
    let mut row = 0;
    calendar.province = super::Province::from_str(&get_meta("Province", row)?)?;
    row += 1;
    calendar.info.description = get_meta("Description", row)?.to_string();
    row += 1;
    calendar.info.created = get_meta("Created", row)?
        .parse()
        .unwrap_or_else(|_| chrono::Local::now());
    row += 1;
    calendar.info.creation = get_meta("Creation", row)?.to_string();
    row += 1;

    let range = workbook.worksheet_range("Holydays")?;
    let headers = range
        .headers()
        .ok_or(CalendarError::new("missing headers"))?;
    debug!("headers are {headers:?}");
    for (row_num, row) in range.rows().enumerate() {
        if row_num == 0 {
            continue;
        } // skip header row
        match holyday_from_row(row, &headers) {
            Err(err) => {
                println!("Error found in spreadsheet at row {row_num}: {err}");
                if ss_err.is_none() {
                    ss_err = Some(CalendarError::new(&format!(
                        "spreadsheet read error ({row_num}): {err}"
                    )));
                }
            },
            Ok(holyday) => calendar.add(&holyday),
        }
    }

    match ss_err {
        None => {
            //debug!("calendar is {calendar:?}");
            Ok(calendar)
        },
        Some(err) => Err(err),
    }
}

/** `hpyday_from_row` derives a [Holyday] from a row of the spreadsheet */
pub fn holyday_from_row(row: &[Data], headers: &Vec<String>) -> Result<Holyday, CalendarError> {
    let mut holyday = Holyday::default();
    // debug!("\n{} ", row.len());
    for (col, head) in headers.iter().enumerate() {
        let val = &row[col];
        // debug!(" [({row_num}:{col}) {head}: {val}], ");
        match head.as_str().trim() {
            "Title" => {
                holyday.title = val
                    .as_string()
                    .ok_or(CalendarError::new("need to specify title"))?
                    .trim()
                    .to_string();
            },

            "Class" => {
                holyday.class = super::HolydayClass::from_str(
                    val.as_string()
                        .ok_or(CalendarError::new("need to specify class"))?
                        .trim(),
                )
                .map_err(CalendarError::from_error)?
            },

            "Calculation" => {
                let kind = val
                    .as_string()
                    .ok_or(CalendarError::new("need to specify title"))?
                    .trim()
                    .to_string();
                let v1 = row[col + 1]
                    .as_string()
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                let v2 = row[col + 2]
                    .as_string()
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                holyday.date_cal = DateCal::new_from_strings(&kind, &v1, &v2)?;
            },
            "Month" | "Day" => {},
            "Transfer" => {
                holyday.transfer = super::TransferType::from_str(
                    val.as_string()
                        .ok_or(CalendarError::new("need to specify transfer"))?
                        .trim(),
                )?;
            },

            "Tag" => {
                holyday.tag = val.as_string().unwrap_or_default().trim().to_string();
            },

            "Death" => {
                holyday.death = val.as_string().unwrap_or_default().trim().to_string();
            },

            "Eve" => {
                holyday.has_eve = val
                    .as_string()
                    .unwrap_or_default()
                    .trim()
                    .to_ascii_lowercase()
                    == "eve";
            },

            "References" => {
                // only Wikipedia for now FUTURE
                holyday.refs = val
                    .as_string()
                    .unwrap_or_default()
                    .trim()
                    .to_string()
                    .split('|')
                    .map(|r| Reference::new(Wikipedia, r.trim().to_string()))
                    .collect();
            },
            "Attributes" => {
                let attr_str = val.as_string().unwrap_or_default().trim().to_string();
                let attr_strs: Vec<_> = attr_str
                    .split('|')
                    .filter(|a| !a.is_empty())
                    .map(|a| {
                        super::MainAttribute::from_str(a.trim()).map_err(CalendarError::from_error)
                    })
                    .collect();
                let attrs = attr_strs
                    .into_iter()
                    .flatten()
                    .collect::<Vec<MainAttribute>>();
                holyday.main.extend(attrs);
            },
            _ => eprintln!("invalid header {}", &head),
        }
    }
    Ok(holyday)
}
impl From<calamine::XlsxError> for super::CalendarError {
    fn from(err: calamine::XlsxError) -> Self { Self::new(&format!("spreadsheet error: {err}")) }
}

impl From<strum::ParseError> for super::CalendarError {
    fn from(err: strum::ParseError) -> Self {
        Self::new(&format!("spreadsheet parse error: {err}"))
    }
}
