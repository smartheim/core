#!/bin/bash -e

killall ohx-core > /dev/null
killall ohx-serve > /dev/null
killall ohx-auth > /dev/null
killall ohx-ruleengine > /dev/null

cargo build
./target/debug/ohx-core &
./target/debug/ohx-serve &
./target/debug/ohx-auth &
./target/debug/ohx-ruleengine &