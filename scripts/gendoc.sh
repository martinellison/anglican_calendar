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
cd $BASE/process_data
echo "generating process data documentation..."
cargo +nightly doc --open
echo "documentation run"
