#!/bin/bash

export $(cat .env)
printenv
exec ./easy_park
