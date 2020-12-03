#!/usr/bin/env bash

# generate all calendars
YEAR=${1-2021}

export BASE=$(git rev-parse --show-toplevel)
if [[ "$BASE" == "" ]]; then
    echo "need to be in the git repository"
    exit 1
fi
cd $BASE
if [[ ! -d $BASE/data/cals ]]; then
    mkdir -p $BASE/data/cals
fi
PROVS="aca acc acsa all bcp cofe ecusa hkskh"
for P in $PROVS; do
    EDFILE=$BASE/data/final/$P.data
    if [[ -f $EDFILE ]]; then
        echo "generating calendar for" $P "<<<"
        target/debug/anglican_calendar -c $EDFILE -i $BASE/data/cals/$P-$YEAR.ical -y $YEAR -u ang-alpha -r $BASE/data/reports/$P-$YEAR.html
        RES=$?
        if [[ $RES != 0 ]]; then
            echo "failed" $RES
            exit 2
        fi
    fi
done
