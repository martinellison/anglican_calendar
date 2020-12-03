#!/usr/bin/env bash

# copy files into GitHub repo so we can push them and make a new release
LOCAL=$HOME/git/anglican_calendar
GITHUB=$HOME/extgit/anglican_calendar

echo "deleting targets..."
rm -rf $LOCAL/*/target
rm -rf $GITHUB/*/target

echo "copying metadata..."
cp $LOCAL/Cargo.toml $GITHUB
cp $LOCAL/README.md $GITHUB
echo "copying src..."
cp -R $LOCAL/src $GITHUB
echo "copying scripts..."
cp -R $LOCAL/scripts $GITHUB
echo "copying edit_data..."
cp -R $LOCAL/edit_data $GITHUB
#rm -rf $GITHUB/edit_data
# echo "copying process_data..."
# cp -R $LOCAL/process_data $GITHUB
rm -rf $GITHUB/process_data
echo "copying final data..."
cp -R $LOCAL/data/final $GITHUB/data
echo "copying reports..."
cp -R $LOCAL/data/reports $GITHUB/data
echo "copying cals ..."
cp -R $LOCAL/data/cals $GITHUB/data

echo "make sure that files are tagged"
