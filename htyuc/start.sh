#!/bin/sh
set -x

echo "----------------------------" >> htyuc.log
echo "$(date)" >> htyuc.log
echo "----------------------------" >> htyuc.log

nohup cargo run >> htyuc.log &

