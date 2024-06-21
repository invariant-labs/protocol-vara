#!/bin/bash

npm run node:local &
# node can crash if it recieves a message too early
sleep 5
npm run test:local
test_status=$?

npm run node:local:stop
exit $test_status