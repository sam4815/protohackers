#!/bin/bash

HOST="0.0.0.0"
PORT=8080

# Open a persistent connection using a subshell
{
    echo '{"request":"get","queues":["queue1","queue2"],"wait":true}'
    echo '{"request":"delete","id":12345}'
    echo '{"request":"abort","id":12345}'
    echo '{"request":"put","queue":"queue1","job":{"lol": 1, "bob": 2},"pri":123}'
    sleep 1
} | nc $HOST $PORT
