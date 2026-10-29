# StremioNative

[![CI](https://github.com/Morpheus235/stremio-native/actions/workflows/ci.yml/badge.svg)](https://github.com/Morpheus235/stremio-native/actions/workflows/ci.yml)
[![Download](https://img.shields.io/badge/download-ipk-6e4ee8)](https://morpheus235.github.io/stremio-native/)

An unofficial native [Stremio](https://www.stremio.com) client for rooted LG webOS TVs. It's
written in Rust with a small C boot shim, draws its own UI with OpenGL ES and plays through the
TV's hardware decoders instead of the web runtime. Developed and tested on an LG C1 (webOS 6).

Not affiliated with Stremio.

<p align="center">
  <img src="docs/screenshots/player.jpg" width="98%" alt="The player">
  <img src="docs/screenshots/home.jpg" width="49%" alt="Home">
  <img src="docs/screenshots/movie.jpg" width="49%" alt="A movie's page">
  <img src="docs/screenshots/discover.jpg" width="49%" alt="Discover">
  <img src="docs/screenshots/list.jpg" width="49%" alt="Discover as a list">
</p>
<p align="center"><sub>Screenshots use the Public Domain Movies addon and a generated test video.</sub></p>

## Install

You need [Homebrew Channel](https://github.com/webosbrew/webos-homebrew-channel) on the TV.
A rooted TV is recommended, because root gives you:

- no Developer Mode session to keep renewing;
- ssh access for logs and debugging;
- working video on some older chip families. There, Developer Mode's sandbox hides the video
  device the player needs, and the app offers a repair that needs root.

Developer Mode without root is supported by the code but untested so far.

1. In Homebrew Channel, open **Settings → Add repository** and enter
   `https://morpheus235.github.io/stremio-native/repo.json`
2. Install **StremioNative Beta**. Once there is a stable release, **StremioNative** appears
   there too, as a separate app that installs beside the Beta. Updates arrive through the same
   repository.

The packages (ipk + sha256) are on the [download page](https://morpheus235.github.io/stremio-native/) and the [releases page](https://github.com/Morpheus235/stremio-native/releases).

## Platform support

| | Status |
| --- | --- |
| LG C1, webOS 6 (rooted) | Developed and tested on it daily |
| Other LG TVs, webOS 4.4 – 11 | The package passes `webosbrew-ipk-verify` for every release from 4.4.2 to 11.2.0, checked in CI against webosbrew's firmware data (static, so a strong hint, not proof). Not yet tested on such a TV. |
| Developer Mode (unrooted) | Supported by the code, untested. Some older chip families need root for video. |
| Build hosts | macOS (arm64, x86_64) and Linux aarch64, where the webOS NDK is published |

**CI** ([`.github/workflows`](.github/workflows)):
- `ci.yml` runs on every push and pull request. It runs the host tests, the shipping feature set,
  the packaging selftests and an ARM compile check, then does a full package build on an arm64
  runner and checks it with `webosbrew-ipk-verify` (it fails on an incompatibility).
- `release.yml` runs on a `v*` tag. It builds, tests and publishes the release.
- `hbrepo.yml` publishes the Homebrew Channel repository and the download page on GitHub Pages
  whenever they change on `main`. After a release, run it by hand so it lists the new one.

## Features

- Stremio sign-in, your account's addons, Continue Watching, Library, Search, Live TV with an
  optional TV guide (now / next from a public XMLTV guide)
- Hardware playback with probing before load; HLS; Dolby Vision Profile 7 played as 8.1
- Audio the TV can't decode (TrueHD, DTS, FLAC, PCM…) converted in software. Under
  **Settings → Multichannel audio**, 5.1 is sent as Dolby Digital (the default, on every output),
  Dolby Digital Plus, PCM, stereo or Pro Logic II.
- Text subtitles through libass (SRT, WebVTT, ASS/SSA, embedded fonts), plus addon subtitles
- Skip intro/recap/credits, CH +/- to jump between chapters, Up Next
- English, Deutsch, Español, Français, Italiano, Беларуская
- Remote control mapping: [docs/remote-keys.md](docs/remote-keys.md)

Status and plans: [docs/roadmap.md](docs/roadmap.md).

## Building

```sh
rustup toolchain install nightly --component rust-src
make setup-env                     # webOS NDK into ~/webos-ndk (or WEBOS_SDK=<sdk root>)
make ipk                           # debug build: build/web.stremio.native.app.debug_<ver>_arm.ipk
make RELEASE=1 FLAVOR=beta ipk     # StremioNative Beta, version X.Y.<UTC date>
make RELEASE=1 FLAVOR=stable ipk   # what a release publishes
```

The NDK is published for macOS and linux-aarch64. `make ipk` also builds the bundled FFmpeg
(`ci/build-ffmpeg.sh`) and libass (`ci/build-libass.sh`, needs `cmake` and `pkg-config`). The
package is reproducible: the same inputs give the same sha256 (`ci/mkipk.py --verify`).

Host tests (no NDK needed):

```sh
# Debian/Ubuntu: apt-get install libsdl2-dev libsdl2-ttf-dev libgl-dev
CARGO_INCREMENTAL=0 cargo test --workspace
make check                         # host tests + packaging selftests
```

Releases are cut by pushing a `vX.Y.Z` tag on `main` (a suffixed tag such as `v0.2.0-beta.1`
publishes the Beta IPK as a GitHub prerelease). You can also run **Actions → Release** and enter
that tag on the branch you want to publish; CI builds the Beta app and attaches the IPK and SHA-256
to Releases. The Homebrew Channel repository is refreshed by hand for now. Details in
[docs/roadmap.md](docs/roadmap.md#releases).

## Layout

| Path | What |
| --- | --- |
| `crates/base` | event log, file locations, spawn discipline, dlopen seam, frame instruments |
| `crates/machine` | state machines, effects, the present gate, idle wake, motion |
| `crates/net` | HTTP(S) over the TV's libcurl, TLS fallbacks, DNS pins |
| `crates/platform` | webOS services, device capabilities, i18n, image cache, storage |
| `crates/gfx` | GL ES renderer, text, SVG, images |
| `crates/ui` | theme, layout, focus engine, widgets |
| `crates/media` | demux, HLS, probing, audio conversion, subtitles, the Starfish player |
| `crates/stremio` | the addon protocol, the Stremio account API, skip markers |
| `crates/screens` | the screens and the stores they read |
| `crates/app` | the application core: boot, frame loop, webOS port, workers |
| `crates/storage` | the private storage helper process |
| `native/` | C boot/crash shim, Starfish/ACB seam, libass and audio-decoder glue, SVG rasterizer |
| `locales/`, `assets/`, `pkg/` | translations, icons, fonts, the app manifest |

## Built on PlxNative

StremioNative is built on [PlxNative](https://github.com/GLinnik21/plx-native), the native Plex
client for LG webOS. Its renderer, UI toolkit, focus engine, network transport, Starfish playback
seam, FFmpeg demuxer and subtitle pipeline are the foundation this app was ported from. Thank you
to its author and contributors.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE), [NOTICE](NOTICE), [docs/legal.md](docs/legal.md) and
[PRIVACY.md](PRIVACY.md).
