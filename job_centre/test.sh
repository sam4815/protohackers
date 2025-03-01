#!/bin/bash

HOST="0.0.0.0"
PORT=8080

# Open a persistent connection using a subshell
{
    echo '{"request":"put","queue":"queue1","job":{"title":"example-job"},"pri":123}'
    echo '{"request":"get","queues":["queue1"]}'
    echo '{"request":"abort","id":12345}'
    echo '{"request":"get","queues":["queue1"]}'
    echo '{"request":"delete","id":12345}'
    echo '{"request":"get","queues":["queue1"]}'
    echo '{"request":"get","queues":["queue1"],"wait":true}'
    sleep 1
} | nc $HOST $PORT
