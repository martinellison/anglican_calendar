#!/usr/bin/env bash
export BASE=$(git rev-parse --show-toplevel)
if [[ "$BASE" == "" ]]; then
    echo "need to be in the git repository"
    exit 1
fi
cd $BASE
echo "generating documentation..."
cargo +nightly doc --open --document-private-items --workspace
echo "documentation run"
