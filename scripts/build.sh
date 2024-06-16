#!/usr/bin/env bash
TARG=$1
export BASE=$(git rev-parse --show-toplevel)
if [[ "$BASE" == "" ]]; then
    echo "need to be in the git repository"
    exit 1
fi
cd $BASE
case "$TARG" in
    release)
        TARGOPT="--release"
        echo "building release..."
        ;;
    debug | "")
        TARGOPT=""
        echo "building debug..."
        ;;
    *)
        echo "unknown target" $TARG
        exit 1
esac
echo 'building...'
cargo build $TARGOPT
if [[ "$TARG" == "release" ]]; then
    mkdir -p $BASE/bin
    TARGDIR=$BASE/target/release
    mv $TARGDIR/calendar $TARGDIR//edit_data $TARGDIR/reports $BASE/bin
fi

echo "build complete"
