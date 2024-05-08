#!/bin/bash

npm run node:local &
# node can crash if it recieves a message too early
sleep 2
npm run test:local &
test_pid=$!

wait $test_pid
test_status=$?
npm run node:local:stop
exit $test_status