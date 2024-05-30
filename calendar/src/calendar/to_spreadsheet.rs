/*! outputs a [super::Calendar] to a spreadsheet */
use super::{CalendarError, DateCal, DateCalDiscriminants};
use rust_xlsxwriter::{Color, Format, FormatBorder, Workbook, XlsxError};
use std::path::Path;
use strum::{EnumCount, IntoEnumIterator, VariantArray, VariantNames};

impl super::Calendar {
    /** `write_to_spreadsheet` writes a [super::Calendar] to a spreadsheet */
    pub fn write_to_spreadsheet(&self, file: &Path) -> Result<(), CalendarError> {
        let mut workbook = Workbook::new();
        let bold_format = Format::new().set_bold();
        let advice_format = Format::new().set_italic().set_font_color(Color::Red);
        let long_format = Format::new()
            .set_bold()
            .set_text_wrap()
            .set_border(FormatBorder::Thin);
        let int_format = Format::new().set_num_format("0");
        let worksheet = workbook.add_worksheet();
        let mut row = 0;

        worksheet.set_name("Calendar")?;
        worksheet.write_with_format(row, 0, "Province", &bold_format)?;
        worksheet.write_with_format(
            row,
            2,
            &format!(
                "province of the Anglican Communion, valid values: {}",
                super::Province::VARIANTS.join(", ")
            ),
            &advice_format,
        )?;
        worksheet.write(row, 1, format!("{:?}", self.province))?;
        row += 1;
        worksheet.write_with_format(row, 0, "Description", &bold_format)?;
        worksheet.write(row, 1, self.info.description.to_string())?;
        worksheet.write_with_format(
            row,
            2,
            "description of this calendar (any text)",
            &advice_format,
        )?;
        row += 1;
        worksheet.write_with_format(row, 0, "Created", &bold_format)?;
        worksheet.write(row, 1, self.info.created.to_string())?;
        row += 1;
        worksheet.write_with_format(row, 0, "Creation", &bold_format)?;
        worksheet.write(row, 1, self.info.creation.to_string())?;
        worksheet.write_with_format(
            row,
            2,
            "author/creator of this calendar (any text)",
            &advice_format,
        )?;

        row += 2;
        worksheet.write_with_format(
            row,
            3,
            "For the holy days in this calendar,\nplease see the \"Holydays\" tab in this \
             spreadsheet.\n\nFor advice on changing the Holydays, see the \"Advice\" tab.",
            &long_format,
        )?;
        row += 1;
        worksheet.autofit();
        worksheet.set_column_width(2, 20)?;

        let worksheet = workbook.add_worksheet();
        worksheet.set_name("Advice")?;
        let mut row = 0;
        worksheet.write_with_format(row, 0, "Title", &bold_format)?;
        worksheet.write_with_format(row, 2, "name of this Holyday", &advice_format)?;

        row += 1;
        worksheet.write_with_format(row, 0, "Class", &bold_format)?;
        let worksheet = worksheet.write_with_format(
            row,
            2,
            &format!(
                "class of holyday, valid values (in increasing order): {}",
                super::HolydayClass::VARIANTS.join(", ")
            ),
            &advice_format,
        )?;
        row += 1;
        worksheet.write_with_format(row, 0, "Calculation", &bold_format)?;
        worksheet.write_with_format(row, 2, "calculation of holyday", &advice_format)?;
        for dc in 0..super::DateCal::COUNT {
            row += 1;
            worksheet.write_with_format(row, 1, DateCal::VARIANTS[dc], &bold_format)?;
            worksheet.write_with_format(
                row,
                2,
                DateCalDiscriminants::VARIANTS[dc].description(),
                &advice_format,
            )?;
        }
        row += 1;
        worksheet.write_with_format(row, 0, "Month", &bold_format)?;
        worksheet.write_with_format(
            row,
            2,
            "month/calculated relative of this Holyday",
            &advice_format,
        )?;
        row += 1;
        worksheet.write_with_format(row, 0, "Day", &bold_format)?;
        worksheet.write_with_format(
            row,
            2,
            "day of month/relative date for this Holyday",
            &advice_format,
        )?;
        row += 1;
        worksheet.write_with_format(row, 0, "Transfer", &bold_format)?;
        worksheet.write_with_format(
            row,
            2,
            &format!(
                "transfer on clash, valid values: {}",
                super::TransferType::VARIANTS.join(", ")
            ),
            &advice_format,
        )?;
        row += 1;
        worksheet.write_with_format(row, 0, "Tag", &bold_format)?;
        worksheet.write_with_format(row, 2, "tag of this Holyday", &advice_format)?;
        row += 1;
        worksheet.write_with_format(row, 0, "Death", &bold_format)?;
        worksheet.write_with_format(
            row,
            2,
            "death date of worthy (any or blank)",
            &advice_format,
        )?;
        row += 1;
        worksheet.write_with_format(row, 0, "Eve", &bold_format)?;
        worksheet.write_with_format(row, 2, "'eve' if required", &advice_format)?;
        row += 1;
        worksheet.write_with_format(row, 0, "References", &bold_format)?;
        worksheet.write_with_format(
            row,
            2,
            "relevant Wikipedia articles separated by '|'",
            &advice_format,
        )?;
        row += 1;
        worksheet.write_with_format(row, 0, "Attributes", &bold_format)?;
        worksheet.write_with_format(
            row,
            2,
            &format!(
                "attributes of worthy separated by '|', valid values: {}",
                super::MainAttribute::VARIANTS.join(", ")
            ),
            &advice_format,
        )?;
        row += 1;
        worksheet.autofit();

        let worksheet = workbook.add_worksheet();
        worksheet.set_name("Holydays")?;
        worksheet.set_header("Holy days in the Calendar");

        let mut col = 0;
        worksheet.write_with_format(0, col, "Title", &bold_format)?;
        col += 1;
        worksheet.write_with_format(0, col, "Class", &bold_format)?;
        col += 1;
        worksheet.write_with_format(0, col, "Calculation", &bold_format)?;
        col += 1;
        worksheet.write_with_format(0, col, "Month", &bold_format)?;
        col += 1;
        worksheet.write_with_format(0, col, "Day", &bold_format)?;
        col += 1;
        worksheet.write_with_format(0, col, "Transfer", &bold_format)?;
        col += 1;
        worksheet.write_with_format(0, col, "Tag", &bold_format)?;
        col += 1;
        worksheet.write_with_format(0, col, "Death", &bold_format)?;
        col += 1;
        worksheet.write_with_format(0, col, "Eve", &bold_format)?;
        col += 1;
        worksheet.write_with_format(0, col, "References", &bold_format)?;
        col += 1;
        worksheet.write_with_format(0, col, "Attributes", &bold_format)?;
        col += 1;
        for (index, day) in self.holydays.iter().enumerate() {
            let mut col = 0;
            worksheet.write((index + 1) as u32, col, day.title())?;
            col += 1;
            worksheet.write((index + 1) as u32, col, day.class().to_string())?;
            col += 1;
            let date_cal = day.date_cal();
            worksheet.write((index + 1) as u32, col, date_cal.to_string())?;
            col += 1;
            match date_cal {
                super::DateCal::Easter | super::DateCal::Advent | super::DateCal::AdventNext => {
                    col += 2;
                },
                super::DateCal::After { date, rel } => {
                    // note: loss of information for some date classes
                    worksheet.write((index + 1) as u32, col, date.to_string())?;
                    col += 1;
                    worksheet.write_number_with_format(
                        (index + 1) as u32,
                        col,
                        rel,
                        &int_format,
                    )?;
                    col += 1;
                },
                super::DateCal::Next {
                    date,
                    day_of_week: _,
                } => {
                    let (month, day) = date.try_month_and_day()?;
                    worksheet.write_number_with_format(
                        (index + 1) as u32,
                        col,
                        month,
                        &int_format,
                    )?;
                    col += 1;
                    worksheet.write_number_with_format(
                        (index + 1) as u32,
                        col,
                        day,
                        &int_format,
                    )?;
                    col += 1;
                },
                super::DateCal::NextSunday { date } => {
                    let (month, day) = date.try_month_and_day()?;
                    worksheet.write_number_with_format(
                        (index + 1) as u32,
                        col,
                        month,
                        &int_format,
                    )?;
                    col += 1;
                    worksheet.write_number_with_format(
                        (index + 1) as u32,
                        col,
                        day,
                        &int_format,
                    )?;
                    col += 1;
                },
                super::DateCal::Fixed { month, day } => {
                    worksheet.write_number_with_format(
                        (index + 1) as u32,
                        col,
                        month,
                        &int_format,
                    )?;
                    col += 1;
                    worksheet.write_number_with_format(
                        (index + 1) as u32,
                        col,
                        day,
                        &int_format,
                    )?;
                    col += 1;
                },
            }
            worksheet.write((index + 1) as u32, col, day.transfer().to_string())?;
            col += 1;
            worksheet.write((index + 1) as u32, col, day.tag().to_string())?;
            col += 1;
            worksheet.write((index + 1) as u32, col, day.death())?;
            col += 1;
            worksheet.write(
                (index + 1) as u32,
                col,
                day.has_eve().then_some("eve").unwrap_or_default(),
            )?;
            col += 1;
            let refs = day
                .refs()
                .iter()
                .map(|r| r.article.to_string())
                .collect::<Vec<_>>()
                .join("|");
            worksheet.write((index + 1) as u32, col, &refs)?;
            col += 1;
            let mains = day
                .main()
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>()
                .join("|");
            worksheet.write((index + 1) as u32, col, &mains)?;
            col += 1;

            /* TODO    other? description? */
        }
        worksheet.autofit();
        workbook
            .save(file)
        //    .map_err(|e| CalendarError::new(&format!("spreadsheet error {e}")))
       ?;
        Ok(())
    }
}
impl From<XlsxError> for super::CalendarError {
    fn from(err: XlsxError) -> Self { Self::new(&format!("spreadsheet error {err}")) }
}
