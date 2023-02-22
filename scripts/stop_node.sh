#!/bin/bash

# Get the PID of the Substrate node
PID=$(pgrep kaioshin)

if [ ! -z "$PID" ]; then
  # If the PID is not empty, kill the Substrate node
  kill $PID
fi
