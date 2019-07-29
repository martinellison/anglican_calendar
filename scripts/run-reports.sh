#!/usr/bin/env bash
OPT=$*
if [[ "$OPT" == "" ]] ; then
    echo "must specify option, try -h"
    exit 1
fi
export BASE=$(git rev-parse --show-toplevel)
if [[ "$BASE" == "" ]]
then
    echo "need to be in the git repository"
    exit 1
fi
cd $BASE
INF=""
for F in `ls $BASE/data/final`; do
    if [[ "$F" != "all.data" ]] ; then
        INF="$INF -i $BASE/data/final/$F"
    fi
done
echo "infiles are" $INF
$BASE/reports/target/debug/reports $OPT $INF
echo "reports generated"
