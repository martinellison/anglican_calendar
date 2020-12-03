#!/usr/bin/env bash
# open editor VSC
export BASE=$(git rev-parse --show-toplevel)
if [[ "$BASE" == "" ]]; then
    echo "need to be in the git repository"
    exit 1
fi
cd $BASE
/usr/bin/code $BASE/anglican_calendar.code-workspace &
