# Anglican calendar #

The purpose of this program is to calculate the holy days of a calendar
of an Anglican church for a given year.

The program could be potentially be extended to operate for the
calendar of the Book of Common Prayer, and for the calendars of other
churches.

Warning: **this program is currently in alpha status and may result in
incorrect data.** Please raise any issues on GitHub.

Also note:

* The data used to calculate holy days and dates has been derived from
  Wikipedia and may be incorrect.
* Each national Church of the Anglican Communion has its own calendars
  including local holy days, and differs from the calendars of other
  national Churches. Also, dioceses and local churches have discretion
  to vary calendars, for example upgrading, downgrading, adding or
  dropping holy days, moving holy days from a weekday to a Sunday and adding
  the dedication day of a church to the local calendar, so the
  calendar observed by your local church may differ from the output of
  these programs.
* Disputes over Church calendars seem to have historically caused more
  heated discussion than any other topic ever. Please do not make this
  a cause of dissention.
* Please raise any issues on GitHub.
* Some of the following requires technical knowledge at some level.
* There are several calendar projects on GitHub for the Tridentine
  calendar: [here](https://github.com/paucazou/theochrone) and
  [here](https://github.com/joe-antognini/tridentine_calendar). This
  project is not related to either of these.
  
## How to do

This section describes some of the most common things to do and some
guidelines on how to do them.

* How to load the holy days of a Church calendar into your calendar
  program (e.g. Google Calendar) and how to cancel these holy days.
* How to generate the holy days for a new year.
* How to build the code.
* How to modify a calendar or create a new calendar.
* How to modify the code.

### How to load the holy days of a Church calendar into your calendar

*Create a new test calendar in your calendar system for just your
Church calendar, and import the holy days into this new
calendar. Then, if you want to undo the load and remove all the holy
days, you can just delete the test calendar. This is just an alpha
version after all.*

There are generated calendars in the `data/cals` directory. Most
calendar programs have some way of loading these files into your
calendar. You want to load a file with a name like `cofe-2019.ical`.

Note that there may be a way to subscribe to a calendar over the
internet, so if someone can set this up it may be the best solution.

Note: aca = Anglican Church of Australia, cofe = Church of England,
ecusa = Episcopal Church of the United States of America, hkskh =
Anglican Church of Hong Kong.

### How to cancel these holy days out of your camera

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
   form, so you will need to build the conversioin program from the
   source code (see the next section). In future, there will be
   pre-built executables for you to download.
2. Ensure that you have a suitable calendar data file in the format
   required by this program. These can be found in the `data/final`
   directory. If you need a different calendar, see the information
   below to create your own.
3. Use the `anglican_calendar` executable to create a calendar and the
   associated deletion file. Running this executable with `--help`
   will describe the options. 
   
   The execution line will be something like `./anglican_calendar -c
data/final/cofe.data -i data/cals/cofe-2019.ical -d
data/cals/cofe-del-2019.ical -y 2019 -u ang-alpha` 

Replace `cofe` and `2019` in the above as required.

The `-u` parameter is to provide a unique identifier for each holy day to
the calendar system (e.g. Google Calendar) so that your calendar app
can delete the correct entries using the deletion file (see "How to
cancel..." above).

### How to build the code

Systems administration  knowledge required.

1. The code is written in the rust programming language, so you will
   need to install the rust tool chain.
2. Clone this repository from GitHub.
3. Build the executables. See the `scripts/build.sh` script for an
   idea of how to do this.

### How to modify a calendar or create a new calendar

Systems administration knowledge required. Also, this is a mess and
unnecessarily difficult to understand.

Your options are:

* modify the file in the `data/final` directory. At the moment, the
   only documentation for this format is in the source code for the
   programs. **or**
* use code such as in `scripts/make-data.sh` to derive a calendar data
  file from Wikipedia data. Good luck with this; you will need it.  
  * first replace commas with `@` signs where they separate fields in
  the input data. See files such as `data/original/cofe.txt` for an
  example.
  * you can use the `edit-data` executable to modify a calendar data
  file. This applies some edits (in their own format). It matches
  edits to holy days using the `tag` field i.e. the tag nmust be the
  same on the old holy day entry and the edit modification for the
  edit to work.

### How to modify the code

Application development knowledge required, including the rust
programming language.

The code includes documentation which can be displayed using rust
tools such as `rustdoc`. Please see this documentation for additional
information about the code internals. Please raise issues and pull
requests if you can.

## The functionality of the executable

The main executable generates the iCal files.

The functionality of the executable is performed by library
crates called from the main program, so that other programs can access
the same functions.

### Possible future extensions

The following are not in the initial release, but could be added
later.

* write an HTML file (file path and name) with a report for display and for web applications
or write a plain text file (file path and name).
* options:
  * start year from Advent (default: January 1).
  * options for the cases that *Common Worship* allows (e.g. moving
    certain holy days to a Sunday, date of celebration of Matthias, etc)
* to include  in the calendar: 
  * Sundays e.g. 16th Sunday after Trinity
  * Fridays and other fasts (eves)
  * ember days
  * seasons and martyrs (in colour)
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

####Bumping (transfer) of holy days  

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
  
The calendar files that are input into the program are generated by
the `process_data` program, which inputs calendar data extracted frm
Wikipedia, as listed in the
[list](https://en.wikipedia.org/wiki/List_of_Anglican_Church_calendars)
of Anglican Church calendars. The `process_data` program only reads
the wiki markup for the actual list of holy days, so that needs to be
extracted and placed in a file.
