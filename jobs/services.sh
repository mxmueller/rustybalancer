#!/bin/bash
docker service ls -q | xargs docker service rm
