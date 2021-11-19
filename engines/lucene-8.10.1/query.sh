#!/bin/bash

SCRIPT_PATH=${0%/*}
cd $SCRIPT_PATH && java -cp ${SCRIPT_PATH}/build/libs/search-index-benchmark-game-lucene-1.0-SNAPSHOT-all.jar DoQuery index
