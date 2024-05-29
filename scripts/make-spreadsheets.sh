#!/usr/bin/env bash
// convert calendars from old format into spreadsheets
PROVS="aca acc acsa bcp cofe ecusa hkskh-2021 "
for PROVINCE in $PROVS; do
    edit_data -t -i data/final/$PROVINCE.data -o data/spreads/$PROVINCE.xlsx
done