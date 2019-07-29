#!/usr/bin/env bash
export BASE=$(git rev-parse --show-toplevel)
if [[ "$BASE" == "" ]]
then
    echo "need to be in the git repository"
    exit 1
fi
cd $BASE
echo "generating main documentation..."
cargo +nightly doc --open
for X in process_data edit_data reports; do
    cd $BASE/$X
    echo "generating $X documentation..."
    cargo +nightly doc --open
done
echo "documentation run"
