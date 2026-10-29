# Roadmap

## Done

- **Foundation.** The PlxNative layers, ported as `sn_*` crates: base, machine, net, platform,
  gfx, ui, storage. Plex-specific transport is gone. The bundled-roots TLS fallback applies to any
  HTTPS host, because old TVs lack the roots most addons chain to.
- **Addons.** Manifest, catalog, meta, stream and subtitle resources, with Cinemeta first.
  Catalogs page with `skip` until a short page comes back. Each addon's timeouts and failures are
  shown, never a silently short list. `behaviorHints` are honoured (`proxyHeaders`,
  `notWebReady`, `bingeGroup`), so live-TV addons such as TvVoo play.
- **Viewport-first loading** (`crates/stremio/src/sched.rs`). The nearest request starts first,
  and work scrolled out of view is cancelled.
- **Account.** Sign-in, a kept session, the account's addons, Continue Watching, Library and
  watched state.
- **Screens.**
  - Home: one row per catalog.
  - Discover, Library, Continue Watching, Live TV, Settings.
  - Search: every catalog with the `search` extra, of any type, with an addon filter, a
    loading/empty/error state per row and recent searches.
  - Detail: episodes by season, and each addon's streams with its own loading state.
  - Top-bar and colour-key shortcuts.
  - Live TV's TV guide (Settings → TV guide): now / next on channel cards and in the player,
    from a region's public XMLTV guide matched by channel name (`crates/stremio/src/epg.rs`).
  - Live TV's Recently watched: the channels the account played, newest first, as the first
    choice of the Live TV catalog picker (a channel has no position, so Continue Watching never
    lists it).
- **Player.**
  - Probe before Load: the bundled demuxer reads the container first, so the Load names the
    real codecs.
  - Hardware playback over the Starfish buffer feed. HLS.
  - Dolby Vision Profile 7 is played as 8.1 (`crates/media/src/dovi.rs`).
  - libass text subtitles with embedded fonts, plus addon subtitles.
  - Track choice, Up Next, and progress saved to the account.
  - Skip intro/recap/credits, from chapter names or introdb (by IMDb id, also for `tmdb:` items).
  - CH +/- jumps between chapters.
- **Audio the TV can't decode.**
  - Software decode feeds S16LE 48 kHz PCM: TrueHD, DTS, FLAC, LPCM, Opus, Vorbis, MP2, WMA, ALAC.
  - Multichannel goes out as AC-3 640 kb/s 5.1 by default on every output, eARC included: on the
    C1 (webOS 6) app PCM is mixed down to stereo inside the TV even with eARC and Pass Through
    (issue #3), while a Dolby bitstream reaches the soundbar as 5.1.
  - Settings → Multichannel audio can choose Dolby Digital Plus (E-AC-3 768 kb/s 5.1, for HDMI
    ARC/eARC), multichannel PCM (not confirmed as 5.1 on any model yet), plain stereo, or Pro
    Logic II.
  - Verified on the C1 over optical with all 5.1 channels in place: AC-3, E-AC-3, AAC, TrueHD,
    DTS, FLAC, PCM.
  - webOS 5/6 refuse a DTS Load, so DTS always goes through software.

## Open

- Player freeze on exit at the end of an episode, or on resuming one. Under investigation: the
  previous session's log is kept as `stremionative-events.prev.log`.
- Stereo and Pro Logic II don't cover AC-3/E-AC-3 tracks yet. The bundled FFmpeg has no ac3/eac3
  decoder, so those stay native.
- Automatic could pick stereo for TV speakers, headphones and Bluetooth output.
- 7.1 PCM over eARC (5.1 is the ceiling today).
- A TvVoo channel that starts without sound.
- Contrast pass on the episode list, the selected episode and the stream list.

## Distribution: Homebrew Channel repository

`ci/hbrepo.py` turns a built ipk into a Homebrew Channel repository: `repo.json`, the package
manifest with sha256 and size, and the ipk URL. `.github/workflows/hbrepo.yml` publishes it on
GitHub Pages on every change to it on `main`; a release no longer refreshes it, so it is run by
hand after one. A TV adds `https://morpheus235.github.io/stremio-native/repo.json`
under Settings → Add repository and gets updates as releases appear.

The package itself is built by `make ipk` (`ci/mkipk.py`). It is reproducible and read back with
`--verify`. The launcher art (`pkg/`, plus the badged `pkg/beta/` and `pkg/debug/` sets) is drawn
by `ci/make-icons.py`.

## Releases

Releases are tags on `main`, with no release branch. Bump `pkg/appinfo.json`'s version, then run
`git tag -a vX.Y.Z -m "StremioNative X.Y.Z" && git push origin vX.Y.Z`.

`release.yml` then:
1. checks that the tag matches the packaged version and is on `main`;
2. runs the tests;
3. cross-builds the ipk on an arm64 runner with the pinned webOS NDK;
4. publishes the GitHub release (ipk + sha256);
5. (the Homebrew Channel repository is refreshed by hand, `hbrepo.yml`, while it is on hold).

`ci.yml` runs on every push and pull request. It runs the host tests, the shipping feature set,
the packaging selftests and an ARM compile check, and does a full package build that is read back
but not published.

### Beta

A suffixed tag (`v0.2.0-beta.1`) builds the separate **StremioNative Beta** app instead:
- `FLAVOR=beta`, app id `web.stremio.native.app.beta`;
- version `X.Y.YYMMDDHHm` (the UTC cut time to ten minutes), so at most one beta per ten minutes;
- published as a GitHub prerelease.

The one `repo.json` lists both apps, each pointing at its newest release, so either or both can
be installed.
