#!/usr/bin/env bash

# make a single calendar from its data file
P=$1
YEAR=$2

EDFILE=data/final/$P.data
if [[ -f $EDFILE ]]
then
    echo "generating calendar for" $P "for" $YEAR
    target/debug/anglican_calendar -c $EDFILE -i data/cals/$P-$YEAR.ical -d data/cals/$P-del-$YEAR.ical -y $YEAR -u ang-alpha
    RES=$?
    if [[ $RES != 0 ]]
    then
        echo "failed" $RES
        exit 2
    fi
fi
