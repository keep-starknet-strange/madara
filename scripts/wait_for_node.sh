#!/bin/bash

# Wait for Substrate node to be ready
while ! nc -z localhost 9944; do sleep 3; done
