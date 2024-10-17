#!/bin/bash

cargo build --color always 2>&1 | tee >(sed $'s/\033[[][^A-Za-z]*m//g' > compile-error.txt)

