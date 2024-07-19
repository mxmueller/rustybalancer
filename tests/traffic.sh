#!/bin/bash

PORTS=(34154 46483 50165)
REQUESTS=1000000
CONCURRENT_REQUESTS=100

generate_load() {
  local port=$1
  echo "Shooting on $port ..."
  for ((i=1; i<=CONCURRENT_REQUESTS; i++))
  do
    (
      for ((j=1; j<=REQUESTS; j++))
      do
        curl -s "http://localhost:$port" > /dev/null
      done
    ) &
  done
  wait
}

for port in "${PORTS[@]}"
do
  generate_load $port &
done

wait

echo "done :)"
