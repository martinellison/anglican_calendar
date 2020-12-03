#!/usr/bin/env bash
export CARGO_NAME="martin"
export CARGO_EMAIL="m.e@acm.org"
export BASE=$(git rev-parse --show-toplevel)
if [[ "$BASE" == "" ]]; then
    echo "need to be in the git repository"
else
    cd $BASE
    MACHINE=$(uname -n)
    echo "machine is $MACHINE, setting machine-specific options"
    case $MACHINE in
    edward | pinkipi | xiaomading | xiaosan) ;;
    tarantula)
        export RUST_SRC_PATH="$(rustc --print sysroot)/lib/rustlib/src/rust/src"
        ;;
    esac
    cd $BASE
    rustup default stable
    case $MACHINE in
    xiaosan) $BASE/scripts/update.sh ;;
    esac
    echo "pulling from git..."
    git pull
    echo "updating crates..."
    cargo upgrade --workspace
    cargo update --aggressive
    for D in $(ls); do
        if [[ -f $BASE/$D/Cargo.toml ]]; then
            cd $BASE/$D/
            cargo upgrade --workspace
            cargo update --aggressive
        fi
    done
    echo "fixing..."
    cargo fix --allow-dirty
    cargo fmt
    echo "building..."
    cd $BASE
    export PATH=$PATH:$BASE/scripts:$BASE/target/debug
    $BASE/scripts/build.sh
    echo "built, starting doco build"
    $BASE/scripts/gendoc.sh &
    echo "all done."
fi
