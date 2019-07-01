#!/usr/bin/env bash

LOCAL=$HOME/git/anglican_calendar
GITHUB=$HOME/extgit/anglican_calendar

cp $LOCAL/Cargo.toml $GITHUB
cp $LOCAL/README.md $GITHUB
cp -R $LOCAL/src $GITHUB
cp -R $LOCAL/scripts $GITHUB
cp -R $LOCAL/edit_data $GITHUB
cp -R $LOCAL/process_data $GITHUB
