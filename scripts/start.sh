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
	localhost)
	    ARCH=""
	    ;;
	*)
	    ARCH="+nightly"
	    codium anglican_calendar.code-workspace&
	    ;;
    esac
    cd $BASE
    echo "fixing..."
    cargo $ARCH fix --workspace --allow-dirty --allow-staged
    echo "clipping..."
    cargo $ARCH clippy --fix --all-targets --all-features --allow-dirty --allow-staged --keep-going
    echo "formatting..."
    cargo $ARCH fmt --all
   
    echo "building..."
    cd $BASE
    export PATH=$PATH:$BASE/scripts:$BASE/target/debug
    $BASE/scripts/build.sh
    echo "built, starting doco build"
    $BASE/scripts/gendoc.sh &
    echo "all done."
fi
