#!/usr/bin/env bash
SCRIPT=`realpath $0`
SCRIPTPATH=`dirname $SCRIPT`
cd $SCRIPTPATH

git commit -a -m "Update cheesedata"
git push

cargo run --release
