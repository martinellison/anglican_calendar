#!/usr/bin/env bash

# generate everything from the original data
YEAR=${1-2021}

export BASE=$(git rev-parse --show-toplevel)
if [[ "$BASE" == "" ]]; then
    echo "need to be in the git repository"
    exit 1
fi
cd $BASE
if [[ ! -d $BASE/data/raw ]]; then
    mkdir -p $BASE/data/raw
fi
if [[ ! -d $BASE/data/fix1 ]]; then
    mkdir -p $BASE/data/fix1
fi
if [[ ! -d $BASE/data/final ]]; then
    mkdir -p $BASE/data/final
fi
if [[ ! -d $BASE/data/cals ]]; then
    mkdir -p $BASE/data/cals
fi
PROVS="aca acc acsa bcp cofe ecusa hkskh"
for P in $PROVS; do
    echo "processing for" $P "<<<"
    process_data/target/debug/process_data -b data -o $BASE/data/raw/$P.data -p $P --descr "from wikipedia for $P"
    RES=$?
    if [[ $RES != 0 ]]; then
        echo "failed" $RES
        exit 2
    fi
    echo "adding feasts for" $P "<<<"
    edit_data/target/debug/edit_data -i $BASE/data/raw/$P.data -e $BASE/data/edits/fest.fixes -o $BASE/data/fix1/$P.data -d "added feasts"
    RES=$?
    if [[ $RES != 0 ]]; then
        echo "failed" $RES
        exit 2
    fi
    PROVEDITS=$BASE/data/edits/$P.fixes
    if [[ -f $PROVEDITS ]]; then
        echo "applying fixes for" $P
        edit_data/target/debug/edit_data -i $BASE/data/fix1/$P.data -e $PROVEDITS -o $BASE/data/final/$P.data -d "added local edits"
        RES=$?
        if [[ $RES != 0 ]]; then
            echo "failed" $RES
            exit 2
        fi
    else
        cp $BASE/data/fix1/$P.data $BASE/data/final/$P.data
    fi
done
# process_data/target/debug/process_data -b data -t
# process_data/target/debug/process_data -b data -d

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
echo "all done <<<"
