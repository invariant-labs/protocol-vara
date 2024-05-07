#!/bin/bash

npm run node:local & 
npm run test:local &
test_pid=$!

wait $test_pid
test_status=$?
npm run node:local:stop
exit $test_status