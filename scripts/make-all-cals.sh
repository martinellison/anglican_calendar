#!/usr/bin/env bash

# generate all iCal calendars
YEAR=${1-2024}
TARG=${2-release}

export BASE=$(git rev-parse --show-toplevel)
if [[ "$BASE" == "" ]]; then
    echo "need to be in the git repository"
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
cd $BASE
if [[ ! -d $BASE/data/cals ]]; then
    mkdir -p $BASE/data/cals
fi
# PROVS="aca acc acsa all bcp cofe ecusa hkskh"
PROVS="hkskh-2024 bcp-1662"
for P in $PROVS; do
    EDFILE=$BASE/data/spreads/$P.xlsx
    if [[ -f $EDFILE ]]; then
        echo "generating calendar for" $P "<<<"
           $BINDIR/calendar --calendar "$EDFILE" i-cal --ical $BASE/data/cals/$P-$YEAR.ical --unique "$P$YEAR" --year $YEAR $OPT 
        RES=$?
        if [[ $RES != 0 ]]; then
            echo "failed" $RES
            exit 2
        fi
    fi
done
