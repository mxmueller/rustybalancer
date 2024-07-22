#!/bin/bash
docker ps -aq | xargs docker stop | xargs docker rm
docker rmi $(docker images -q)
docker builder prune --all --force
