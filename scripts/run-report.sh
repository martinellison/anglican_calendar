#!/usr/bin/env bash
# run the report

TARG=$1
export BASE=$(git rev-parse --show-toplevel)
if [[ "$BASE" == "" ]]; then
    echo "need to be in the git repository"
    exit 1
fi
cd $BASE
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
$BINDIR/calendar -c data/spreads/hkskh-2024.xlsx --unique hkskh --year 2024 --report /tmp/hkskh.html
echo "done"
