#!/usr/bin/env bash

build() {
  cargo build --release --locked

  CURR_DIR=$(pwd)
  cd themes/lightdm-webkit-theme-litarvan
  ./build.sh
  cd $CURR_DIR
}

package() {
  sudo install -Dm755 target/release/lightdm-webkit-greeter /usr/bin/lightdm-webkit-greeter
  sudo install -Dm755 target/release/liblightdm_webkit_greeter_webext.so /usr/lib/lightdm-webkit-greeter/liblightdm-webkit-greeter-webext.so

  sudo install -Dm644 data/web-greeter.yml /etc/lightdm/web-greeter.yml
  sudo install -Dm644 data/lightdm-webkit-xgreeter.desktop /usr/share/xgreeters/lightdm-webkit-greeter.desktop

  CURR_DIR=$(pwd)
  cd themes/lightdm-webkit-theme-litarvan
  VERSION=$(cat version)
  sudo cp ./lightdm-webkit-theme-litarvan-$VERSION.tar.gz /usr/share/web-greeter/themes/litarvan/
  cd /usr/share/web-greeter/themes/litarvan/
  sudo tar -xvf lightdm-webkit-theme-litarvan-$VERSION.tar.gz
  cd $CURR_DIR
}

build

package

