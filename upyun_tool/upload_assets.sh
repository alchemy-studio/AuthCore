#!/bin/sh
set -x

CONFIG=${1:-./.upyun_pass}

for FILE in $(ls -1 assets)
do
    upyun_tool --upload $FILE ./assets/$FILE --config $CONFIG
done
