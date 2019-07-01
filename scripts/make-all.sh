#!/usr/bin/env bash
# merge some calendars into one combined calendar   with all the holy days from the calendars. 
PROVS="aca ecusa hkskh" # use cofe as the original, these are all the others
ORIG=cofe

EDITS=""
for P in $PROVS
do
    EDFILE=data/final/$P.data
    edit_data/target/debug/edit_data -a edits -i $EDFILE    
    RES=$?
    if [[ $RES != 0 ]]
    then
        echo $P "failed" $RES
        exit 2
    fi
    EDITS="$EDITS $EDFILE.edits"
done
echo "edits are" $EDITS

edit_data/target/debug/edit_data -i data/final/$ORIG.data -e $EDITS -o data/final/all.data
         

