#!/usr/bin/env bash

# copy files into GitHub repo so we can push them and make a new release
LOCAL=$HOME/git/anglican-calendar
GITHUB=$HOME/extgit/anglican_calendar
if [[ ! -d $LOCAL ]]; then
    echo "cannot find" $LOCAL
    exit 1
fi
if [[ ! -d $GITHUB ]]; then
    echo "cannot find" $GITHUB
    exit 1
fi

echo "deleting targets..."
rm -rf $LOCAL/*/target
rm -rf $GITHUB/*/target

echo "copying metadata..."
cp $LOCAL/Cargo.toml $GITHUB
cp $LOCAL/Cargo.lock $GITHUB
cp $LOCAL/.gitignore$GITHUB
cp $LOCAL/README.md $GITHUB
echo "copying src..."
cp -R $LOCAL/calendar $GITHUB
echo "copying scripts..."
cp -R $LOCAL/scripts $GITHUB
echo "copying edit_data..."
cp -R $LOCAL/edit_data $GITHUB
rm -rf $GITHUB/process_data
echo "copying final data..."
cp -R $LOCAL/data/spreads/* $GITHUB/data/spreads
# echo "copying cals ..."
# cp -R $LOCAL/data/cals $GITHUB/data

echo "make sure that files are tagged"
