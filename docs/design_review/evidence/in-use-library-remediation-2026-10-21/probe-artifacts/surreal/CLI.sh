#!/bin/sh
/home/paul/.local/bin/surreal sql --endpoint http://127.0.0.1:39539 --namespace probe --database "${2:-probe}" --username root --password root --hide-welcome --json < "$1"
