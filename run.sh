#!/usr/bin/env bash
SCRIPT=`realpath $0`
SCRIPTPATH=`dirname $SCRIPT`
cd $SCRIPTPATH

cargo run --release
