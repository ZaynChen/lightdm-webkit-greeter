#!/usr/bin/env bash

if ( ! command -v systemctl >/dev/null 2>&1 ) ; then
  echo "systemd not installed"
  exit 1
fi

if ( ! command -v WebKitWebDriver >/dev/null 2>&1 ) ; then
  echo "webkitgtk not installed"
  exit 1
fi

_pkgname="lightdm-webkit-greeter"

build() {
  cargo build --release --locked

  CURR_DIR=$(pwd)
  cd themes/litarvan
  ./build.sh
  cd $CURR_DIR
}

package() {
  sudo install -Dm0755 "target/release/$_pkgname" "/usr/bin/$_pkgname"
  sudo install -Dm0755 "target/release/libwebext.so" "/usr/lib/webkit-greeter/libwebext.so"

  sudo install -Dm644 data/webkit-greeter.toml /etc/lightdm/webkit-greeter.toml
  sudo install -Dm644 data/lightdm-webkit-xgreeter.desktop /usr/share/xgreeters/lightdm-webkit-greeter.desktop

  CURR_DIR=$(pwd)
  cd themes/litarvan
  _themepkg="*-$(cat version).tar.gz"
  sudo install -Dm0755 $_themepkg -t "/usr/share/webkit-greeter/themes/litarvan/"
  cd /usr/share/webkit-greeter/themes/litarvan/
  sudo tar -xvf $_themepkg
  cd $CURR_DIR
}

build

package
