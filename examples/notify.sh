#!/bin/sh
cat << EOF | nc -Cv 127.0.0.1 6606
login $(hostname)
title: $1
body: $2
send
quit
EOF
