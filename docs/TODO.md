<!-- Required extensions: pymdownx.betterem, pymdownx.tilde, pymdownx.emoji, pymdownx.tasklist, pymdownx.superfences -->
# To do list

 *CW* = *Common Worship* [Rules](https://www.churchofengland.org/prayer-and-worship/worship-texts-and-resources/common-worship/churchs-year/rules) for the Church's Year. See also [rules](#rules) below.

 * add "Eve of" to title of vigils

## 0.2.4

* implemented Eves (Vigils)
    * Eves falling on a Sunday are put back to the Saturday
        * at least for BCP 1662, should this be an option?

## Correctness of existing functions and data

* check/fix calculation for Baptism of Christ [0.2.3 2024 ok]
* Corpus Christi [0.2.3 testing]
  * *per* *CW* Corpus Christi is an optional Festival with its own transfer rule
  * the code has a special `CorpusChristi` class [0.2.3 2024 wrong date and colour]
* ensure there are no saints' days in Holy Week [0.2.3 Annunciation is wrong, others not tested for 2024]
* add tests for Advent Sunday and St George's Day [0.2.3 testing] (and above cases)
* seasons and martyrs (in colour)
  * clashes of holy days
    * correct processing (remove/downgrade)
    * show explanation in report, same as calendar


### Current (0.2.3) issues

* ~~St Irenæus (28 June) should be white, is green~~
* ~~SS Anne and Joachim should be white, is green~~
* ~~SS Mary, Martha & Lazarus should be white, is green~~
* BVM is 8 September (this is optional)
* ~~St John Chrysostom should be white, is green~~
* ~~St Jerome should be white, is green~~
* ~~St Francis Xavier and St Clement of Alexandria should be white not purple~~
* ~~St Ambrose should be white not purple~~
* not sure about some holy days e.g. Festival vs Lesser Festival

### Old (0.2.2) issues

~~Comparing
  1. the output of the current version (0.2.3), run against HKSKH spreadsheet for 2024, and
  1. the calendar published by the HKSKH RERC. Note: this does not directly display the level of each holyday (or does it?).~~

* St Matthias should be martyr  [0.2.3 testing]
* Annunciation — wrong date [0.2.3 testing]
* Maundy Thursday — colour should be white [0.2.3 testing]
* Holy Saturday should be holy day grey?
* Trinity, Corpus Christi, ~~SS Anne and Joachim~~, SS Mary etc, St Ignatius, St Dominic, St Gregory — colour all should be white
* BVM — date etc
* ~~St John Chr. — colour should be white~~
* ~~St Jerome — ?~~
* All Souls', ~~St Francis X, St Clement, St Ambrose — colour all should be white~~

## Enhancements

The following are not in the 0.2.3 release, but could be added
later.

* add Sundays
  * e.g. 16th Sunday after Trinity
  * correct numbering/'Last..'
  * cutoff before Advent/Lent (number of Sundays varies year on year)
* add other days such as 
  * Ember Days
  * week of Christian Unity
  * Fridays and other fasts (eves)
  * ember days
* add other sources for links than Wikipedia (which sources?)
* fix the Book of Common Prayer (BCP) calendar
  * clarify – this means the 1662 BCP 
  * compare against the 1662 BCP and correct the spreadsheet
* options:
  * start year from Advent (default: January 1).
  * options for the cases that *Common Worship* allows (e.g. moving
    certain holy days to a Sunday, date of celebration of Matthias, etc)
* write an HTML file (file path and name) with a report for display and for web applications
or write a plain text file (file path and name).
* multiple languages (e.g. English and Chinese) for the same province/calendar.

## iCal calendars

* fix scripts (make all calendars and make calendar) using spreadsheets
* check that links work
* regenerate and publish

## Other

* improve error processing
  * ~~don't panic, send an error~~ (mainly, anyway)
  * make error reports more friendly to users
* ~~liturgical colours do not match HKSKH calendar, fix (code problem or data?)~~
* add more tests
* ~~get rid of 'other' and 'description' fields~~
* see TODO in code
  * ````find -name *.rs | xargs   grep TODO ```` to find all to-dos

## Data cleanup

Things that need to be done to the data.

* ~~Find any remaining duplicate holydays~~
* ~~changes to HK calendar?~~

## Rules

### Options

These are options allowed by *CW*.

☺ = can be done with the existing spreadsheet.

#### Principal Holy Days

* Epiphany on weekday *may transfer to* (nearby) Sunday, *in which case* Baptism of Christ *transfers*.
* Presentation (Candlemas) *may transfer to* (nearby) Sunday.
* All Saints' *may transfer to* (nearby) Sunday.
* All Souls' if on Sunday *may* be kept on 3 November.
* Corpus Christi *may be* a Festival (*or* what?) ☺
* Blessed Virgin Mary *may be* 15 August (Dormition/Assumption) *or* 8 September (Birth) ☺.

#### Festivals

* Festival *clash* (certain festival Sundays) *transfer to* that Sunday *or* the first available day.
* Dedication of Church and Patron Saint *can be* Festival or Principal Holy Day ☺ *or* following Sunday.
* Most Festivals on Sunday *may* be transferred.

#### Lesser Festivals

* Lesser Festivals 
    * *may be* omitted *or* kept as Commemorations ☺.
    * are *normally* omitted on Principal/Festival/Sunday/Eastertide *or* transferred to the next available day.
* The colour of a Lesser Festival *may be* red/white *or* the seasonal colour.

#### Commemorations

* Commemoration *may be* upgraded to a Festival ☺.
