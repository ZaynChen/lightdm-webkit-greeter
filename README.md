<div align="center">
  <h1><strong>LightDM WebKit Greeter</strong></h1>
  <p>
    <strong>Another LightDM greeter made with WebKitGTK</strong>
  </p>
</div>

LightDM WebKit Greeter for LightDM allows to create themes with web technologies,
the same as the outdated [lightdm-webkit2-greeter][webkit2-greeter]. This project
is ported from [sea-greeter][sea-greeter], with migration from gtk3 and webkit2gtk
to gtk4 and webkitgtk.

## Alternative

[webkit-greeter][webkit-greeter] is based on this project, and support greetd(with hyprland) and lightdm.

## Known issues

There are lots of issues, even not documented here, so it is not recommended to use this greeter yet.
However, it is functional :D

- [x] Multi-monitor support.
    - [x] Add `greeter_comm` JavaScript API
    - [x] Parse `index.yml` to load `secondary.html`
- [ ] Brightness feature support
- [ ] Battery feature support
- [x] Detect theme errors prompt
- [x] Memory management might not be correct; possible memory leaks. (I hope this is fixed)
- [x] Add themes
- [x] Add config

## Dependencies

- libgtk4
- webkitgtk-6.0
- libwebkitgtk-web-extension
- libglib-2.0
- liblightdm-gobject-1

### Build dependencies

- Rust
- Cargo
- npm

## Build and install

```sh
git clone https://github.com/ZaynChen/lightdm-webkit-greeter --recursive
cd lightdm-webkit-greeter
./install.sh
```

## Theme JavaScript API:

The greeter exposes a JavaScript API to themes which they must use to interact with the greeter (in order to facilitate the user login process). For more details, check out the [API Documentation](https://doclets.io/Antergos/lightdm-webkit2-greeter/stable). 

[webkit2-greeter]: https://github.com/Antergos/web-greeter/tree/stable "LightDM WebKit2 Greeter"
[sea-greeter]: https://github.com/JezerM/sea-greeter "Sea Greeter"
[webkit-greeter]: https://github.com/ZaynChen/webkit-greeter "WebKit Greeter"
