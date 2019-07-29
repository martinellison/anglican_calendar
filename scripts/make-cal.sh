#!/usr/bin/env bash

# make a single calendar from its data file
P=$1
YEAR=$2
OPT=$3 # may be empty, could be e.g. -v
if [[ "$P" == "" ]] ; then
    echo "need to provide province"
    exit 1
fi
if [[ "$YEAR" == "" ]] ; then
    echo "need to provide year"
    exit 1
fi

EDFILE=data/final/$P.data
if [[ -f $EDFILE ]]
then
    echo "generating calendar for" $P "for" $YEAR
    target/debug/anglican_calendar -c $EDFILE -i data/cals/$P-$YEAR.ical -r data/reports/$P-$YEAR.html -y $YEAR -u ang-alpha $OPT 
    RES=$?
    if [[ $RES != 0 ]]
    then
        echo "failed" $RES
        exit 2
    fi
fi
