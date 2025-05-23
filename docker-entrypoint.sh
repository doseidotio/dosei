#!/bin/bash

# Forward SIGINT to child process
trap 'kill -INT $child' INT

if [ -z "$DATABASE_URL" ]; then
  service postgresql start
fi

# Start your application in the background
/usr/local/bin/doseid &
child=$!

# Wait for the application to exit
wait $child
