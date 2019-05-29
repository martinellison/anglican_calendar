#!/usr/bin/env bash

# generate everything from the original data

export BASE=$(git rev-parse --show-toplevel)
if [[ "$BASE" == "" ]]
then
    echo "need to be in the git repository"
    exit 1
fi
cd $BASE
PROVS="cofe aca ecusa hkskh"
for P in $PROVS
do
    echo "processing for" $P "<<<"
    process_data/target/debug/process_data -b data -o data/raw/$P.data -p $P -v
    RES=$?
    if [[ $RES != 0 ]]
    then
        echo "failed" $RES
        exit 2
    fi
    echo "adding feasts for" $P "<<<"
    edit_data/target/debug/edit_data -i data/raw/$P.data -e data/edits/fest.fixes -o data/fix1/$P.data 
    RES=$?
    if [[ $RES != 0 ]]
    then
        echo "failed" $RES
        exit 2
    fi
    PROVEDITS=data/edits/$P.fixes
    if [[ -f $PROVEDITS ]]
    then
        echo "applying fixes for" $P
        edit_data/target/debug/edit_data -i data/fix1/$P.data -e $PROVEDITS -o  data/final/$P.data
        RES=$?
        if [[ $RES != 0 ]]
        then
            echo "failed" $RES
            exit 2
        fi
    else
        cp  data/fix1/$P.data  data/final/$P.data
    fi
done
# process_data/target/debug/process_data -b data -t
# process_data/target/debug/process_data -b data -d

for P in $PROVS
do
    EDFILE=data/final/$P.data
    if [[ -f $EDFILE ]]
    then
        echo "generating calendar for" $P "<<<"
        target/debug/anglican_calendar -c $EDFILE -i data/cals/$P-2019.ical -d data/cals/$P-del-2019.ical -y 2019 -u ang-alpha
        RES=$?
        if [[ $RES != 0 ]]
        then
            echo "failed" $RES
            exit 2
        fi
    fi
done
echo "all done <<<"
