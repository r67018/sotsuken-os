#!/bin/bash

EDK2_DIR=~/edk2

# カーネルをコンパイルしてログを記録
if ! cargo build --color always 2>&1 | tee >(sed $'s/\033[[][^A-Za-z]*m//g' >> compile-error.txt); then
  exit 1
fi

# ブートローダーをビルド
cd $EDK2_DIR
source edksetup.sh
build

