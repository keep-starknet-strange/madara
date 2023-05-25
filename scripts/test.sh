#!/usr/bin/env bash

npm run test:wait

if [ $? -eq 0 ]
then
  kill -9 $(lsof -t -i:9944)
  exit 0
else
  kill -9 $(lsof -t -i:9944)
  exit 1
fi
