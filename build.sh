#!/bin/bash

EDK2_DIR=~/edk2

# カーネルをコンパイル
source ~/osbook/devenv/buildenv.sh
CARGO_LOG=$(cargo build --color always 2>&1)
CARGO_STATUS=$?
# ログを記録
echo "$CARGO_LOG" | tee >(sed $'s/\033[[][^A-Za-z]*m//g' >> compile-error.txt)
# コンパイルが失敗なら終了
if [ $CARGO_STATUS -ne 0 ]; then
  exit 1
fi

# ブートローダーをビルド
cd $EDK2_DIR || exit 1
source edksetup.sh
build
