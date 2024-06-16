# Anglican calendar

This project is for generating the Church Calendar for Churches of the Anglican Communion.

It can be used to:

* create a report of a calendar for a year.
* generate a calendar that can be loaded into calendar programs such as Google Calendar.

## Please send feedback

Please send feedback using the GitHub web page. It might not be looked at straight away.

In particular, please raise:

* anything that causes the program to crash.
* any incorrect input data.
* any incorrect output.
* any new or modified calendars, including data for other Provinces that are not currently recorded.

## Version 0.2

This project has been revised to change the main input format to spreadsheets as this is easier to edit.

Old format input files can be converted; see `scripts/make-spreadsheets.sh` for an example script.


## How to do

This section describes some of the most common things to do and some
guidelines on how to do them.

* How to load the holy days of a Church calendar into your calendar
  program (e.g. Google Calendar) and how to cancel these holy days.
* How to generate the holy days for a new year, and print a report.
* How to build the code.
* How to modify a calendar or create a new calendar.
* How to modify the code.

### How to load the holy days of a Church calendar into your calendar

*Create a new test calendar in your calendar system for just your
Church calendar, and import the holy days into this new
calendar. Then, if you want to undo the load and remove all the holy
days, you can just delete the test calendar. This program is just an alpha
version after all.*

There are generated calendars in the `data/cals` directory. Most
calendar programs have some way of loading these files into your
calendar. You want to load a file with a name like `cofe-2019.ical`.

Note: aca = Anglican Church of Australia, cofe = Church of England,
ecusa = Episcopal Church of the United States of America, hkskh =
Anglican Church of Hong Kong.

The `calendar` program also can generate a report of the calendar for a given province and year.

### How to add a new calendar to Google Calendar from URL

You can use the GitHub raw links for each ICAL file to add a new calendar to Google Calendar. This will add a separate calendar which you can toggle or delete as required, without affecting your other calendars.

1. Obtain a link to the calendar of your choice by navigating to the file in the `data/cals` directory in the GitHub web UI, then clicking "Raw". You may want to use a particular commit, in case the file paths change in the future. For example, the C of E 2021 calendar link for d4c502a is:
https://raw.githubusercontent.com/martinellison/anglican_calendar/d4c502a8469f67d7e010927a9876e0b1e1781e01/data/cals/cofe-2021.ical

2. In Google Calendar, navigate to **Other calendars** -> **Add other calendars (+)** -> **From URL**.

3. Enter the URL from step 1 and click **Add calendar**

Google Calendar reloads the calendar from the URL every few hours, so if you use an URL like the following, the events may change:
https://raw.githubusercontent.com/martinellison/anglican_calendar/release/data/cals/cofe-2021.ical

### How to cancel these holy days out of your calendar

*Warning: the following appears not to work with Google Calendar, and
has not been tested on anything else. You are probably better off just
deleting the entire test calendar. You did load the holy days into a
separate test calendar like it says in the previous section, didn't
you?*. The `data/cals` directory also contains files with names like
`cofe-del-2019.ical` (with `del`). These 'should' be able to delete
the entries created by the previous calendar.

### How to generate the holy days for a new year

Some technical knowledge required.

1. At the moment, the programs are only distributed in source code
   form, so you will need to build the conversion program from the
   source code (see the next section). In future, there will be
   pre-built executables for you to download.
2. Ensure that you have a suitable calendar data file in the format
   required by this program. These can be found in the `data/final`
   directory. If you need a different calendar, see the information
   below to create your own.
3. Use the `calendar` executable to create a calendar and the
   associated deletion file. Running this executable with `--help`
   will describe the options.

   The execution line will be something like `./calendar -c
data/final/cofe.data -i data/cals/cofe-2019.ical -d
data/cals/cofe-del-2019.ical -y 2019 -u ang-alpha`

Replace `cofe` and `2019` in the above as required.

The `-u` parameter is to provide a unique identifier for each holy day to
the calendar system (e.g. Google Calendar) so that your calendar app
can delete the correct entries using the deletion file (see "How to
cancel..." above).

The `calendar` program also can generate a report of the calendar for a given province and year, so you can check the spreadsheet. The report is in HTML format, so load it as a file into your favourite web browser to read it. Use the `calendar` program with the `--report my_report.html` option.

### How to build the code

Systems administration  knowledge required.

1. The code is written in the Rust programming language, so you will
   need to install the Rust tool chain.
2. Clone this repository from GitHub.
3. Build the executables. See the `scripts/build.sh` script for an
   idea of how to do this.

### How to modify a calendar or create a new calendar

Systems administration knowledge required. *Rewritten for version 0.2.*

Calendar data is contained in spreadsheets (with a defined format).

To create a new calendar, take a copy of one of the existing spreadsheets and modify it as required.

The spreadsheet has three worksheets:

Worksheet | Purpose
--- | ---
Province | overall data for the calendar
Advice | description of the 'Holydays' worksheet, not read by the program
Holydays | list of holy days

### How to modify the code

Application development knowledge required, including the Rust
programming language.

The code includes documentation which can be displayed using Rust
tools such as `rustdoc`. Please see this documentation for additional
information about the code internals. Please raise issues and pull
requests on GitHub if you can.

## The functionality of the executable

The main executable (`calendar`) generates the iCal files.

The functionality of the executable is performed by library
crates called from the main program, so that other programs can access
the same functions.

## Other executables

The other executables are less important.

### Edit data

`edit_data` is only used for tidying a spreadsheet. You probably will not use this.

### Reports

`reports` was used to generate some reports about comparing more than one calendar. You will not need this.

### Possible future extensions

The file [todos](docs/TODOS.md) contains some possible future extensions that are not in the initial release, but could be added
later.

## Points to note ##

* the same holy day may have slightly different names in different
  province and calendars. Holy Days can have 'tags' which are supposed to
  be consistent across calendars to enable the merger of calendars and
  overriding of holy day details as required.
* some minor holy days may be 'bumped' (transferred) by major holy days. See [the
  rules](https://www.churchofengland.org/prayer-and-worship/worship-texts-and-resources/common-worship/prayer-and-worship/worship-texts-and-resources/common-worship/churchs-year/rules).
  * note: only fixed holy days are moved, except that Patronal and
    Dedication replace Sunday.
  * further discussion of bumping below.
* some holy days may not occur in some years e.g. whether there is, or is
not, a 23rd Sunday after Trinity will depend on the date of Easter in
the year and the date of Advent in the following year.

#### Bumping (transfer) of holy days

The program ensures that dates are bumped correctly [hopefully].

* It processes non-transferable holy days first; then checks all other holy days
  as to whether each should be transferred.
* Holy Days in the calendar are marked by a transfer type. The holy days with
  the more complex rules have their own transfer type. Note: a
  specific calendar may possibly not use the transfer type that is
  normally used for a specific holy day; and this may mean that a new
  calendar with an different bump rule may only be handled after the
  code has been enhanced to cover the new rule.

## Derivation of the data

The calendar files that are input into the program were originally generated from
calendar data extracted from
Wikipedia, as listed in the
[list](https://en.wikipedia.org/wiki/List_of_Anglican_Church_calendars)
of Anglican Church calendars. 

## Development to-do

See [todos](docs/TODO.md) for known problems and possible enhancements.



