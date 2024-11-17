#!/usr/bin/env bash
# make a single calendar from its data file
P=$1
YEAR=$2
TARG=$3
OPT=$4 # may be empty, could be e.g. -v
if [[ "$P" == "" ]] ; then
    echo "need to provide province"
    exit 1
fi
if [[ "$YEAR" == "" ]] ; then
    echo "need to provide year"
    exit 1
fi
case "$TARG" in
    release)
        BINDIR=$BASE/bin
        ;;
    debug | "")
        BINDIR=$BASE/target/debug
        ;;
    *)
        echo "unknown target" $TARG
        exit 1
esac

EDFILE=data/spreads/$P.xlsx
if [[ ! -f $EDFILE ]]
then
    echo "could not find input spreadsheet" $EDFILE
    exit 2
fi
    echo "generating calendar for" $P "for" $YEAR 
    $BINDIR/calendar --calendar "$EDFILE" i-cal --ical /tmp/$P.ical --unique "$P$Y" --year $YEAR $OPT 
    RES=$?
    if [[ $RES != 0 ]]
    then
        echo "failed" $RES
        exit 3
    fi
