# StremioNative — the device build and the package.
#
#   make setup-env                       fetch + relocate the webOS NDK (ci/deps.py, sha256-pinned)
#   make                                 build/stremionative + build/stremionative-storage (ARM)
#   make ipk                             build/<app id>_<version>_arm.ipk for $(FLAVOR)
#   make RELEASE=1 FLAVOR=stable ipk     what a release publishes (release.yml)
#   make check                           host tests + the packaging selftests (no NDK)
#   make selftest                        the packaging selftests alone
#   make icons                           redraw pkg/*.png (ci/make-icons.py, needs Pillow)
#
# FLAVOR picks the install (ci/flavor.py): `debug` (the default, a developer build beside the
# others), `beta` (the prerelease channel: its own app id, tile "StremioNative Beta", version
# X.Y.YYMMDDHHm from pkg/appinfo.json's X.Y and the UTC cut time to ten minutes) or `stable`. stable and beta are
# what other people run, so they are ALWAYS RELEASE=1 builds (`release-guard`): no dev triggers.
#
# The NDK is $(WEBOS_SDK) (default ~/webos-ndk/arm-webos-linux-gnueabi_sdk-buildroot, where
# `make setup-env` puts it). webosbrew publishes it for darwin-arm64, darwin-x86_64 and
# linux-aarch64 only, which is why CI packages on ubuntu-24.04-arm.
#
# The bundled FFmpeg (and, when present, libass) ship inside the package: the player dlopens them
# from the app directory by their `-sn` SONAMEs. By default `make ipk` builds FFmpeg with
# ci/build-ffmpeg.sh into vendor/ffmpeg-prefix; BUNDLED_LIB_DIR=<dir> takes prebuilt libraries
# instead, and a missing required library stops the package with its name (README.md, Building).

.DEFAULT_GOAL := all
.DELETE_ON_ERROR:

FLAVORS := $(shell python3 ci/flavor.py --list)
$(if $(strip $(FLAVORS)),,$(error ci/flavor.py --list produced no flavours — is python3 available?))
FLAVOR ?= debug
$(if $(filter $(FLAVOR),$(FLAVORS)),,$(error unknown FLAVOR "$(FLAVOR)" — one of: $(FLAVORS)))

# The id users get; ci/flavor.py's selftest holds it equal to ci/install-identities.json and
# crates/base/src/paths.rs.
APPID_STABLE := web.stremio.native.app
APPID := $(shell python3 ci/flavor.py $(FLAVOR))
# A beta's package version carries its cut time; computed once per make run (UTC), exported so the
# packager and the version the binary reports name the same cut.
SN_BETA_DATE := $(shell date -u +%Y%m%d%H%M)
export SN_BETA_DATE
IPK_VERSION := $(shell SN_BETA_DATE=$(SN_BETA_DATE) python3 ci/flavor.py --version $(FLAVOR))

WEBOS_SDK ?= $(HOME)/webos-ndk/arm-webos-linux-gnueabi_sdk-buildroot
export WEBOS_SDK
TOOLPREFIX = $(WEBOS_SDK)/bin/arm-webos-linux-gnueabi-
SYSROOT    = $(WEBOS_SDK)/arm-webos-linux-gnueabi/sysroot
# The project's NDK driver (ci/arm-cc.py): the SDK's gcc, with the glibc polyfill archive kept out
# of every link and the link's evidence (map, trace, sha256 receipt) written beside its output.
CC = python3 $(CURDIR)/ci/arm-cc.py

RUST_NIGHTLY ?= nightly
RUST_TARGET  = arm-unknown-linux-gnueabi
# `release` (fat LTO) for anything that ships or is measured; the fast `tvdev` profile (no LTO)
# for an ordinary local debug build. ARM_PROFILE=… on the command line overrides.
ARM_PROFILE ?= $(if $(or $(RELEASE),$(filter-out debug,$(FLAVOR)),$(CI),$(GITHUB_ACTIONS)),release,tvdev)
$(if $(filter release tvdev,$(ARM_PROFILE)),,$(error unknown ARM_PROFILE "$(ARM_PROFILE)" — one of: release tvdev))
ARM_PROFILE_FLAG = $(if $(filter release,$(ARM_PROFILE)),--release,--profile $(ARM_PROFILE))
# A release build carries no dev triggers; it reports the PACKAGE's version (crates/app/build.rs).
RUST_FEATURES = $(if $(RELEASE),--no-default-features,)
RUST_VERSION_ENV = $(if $(RELEASE),SN_VERSION=$(IPK_VERSION),)
# Codegen flags (cortex-a9, no NEON, frame pointers) live in .cargo/config.toml, bound to the
# target, so every cargo invocation gets them; do not set RUSTFLAGS here (it would replace them).
BUILD_STD = -Z build-std=std,panic_unwind

BUILD     = build
RUST_TDIR = target/arm-app
RUST_LIB  = $(RUST_TDIR)/$(RUST_TARGET)/$(ARM_PROFILE)/libstremionative_app.a
# The storage helper is its own package with its own target dir: two cargo invocations sharing one
# would race on the build-std units under `make -j`.
STORAGE_TDIR = target/arm-storage
STORAGE_BIN  = $(STORAGE_TDIR)/$(RUST_TARGET)/$(ARM_PROFILE)/stremionative-storage

# Every C file ships but the libass glue (ass.c, ass_fonts.c belong to the bundled renderer, not the
# application ELF; adec.c is libsnadec.so, below) and the opt-in allocator guard.
SRCS = $(filter-out native/gpdebug.c native/ass.c native/ass_fonts.c native/adec.c,$(wildcard native/*.c)) native/compat/getauxval.c
OBJS = $(patsubst native/%.c,$(BUILD)/obj/%.o,$(SRCS))
NANOSVG = vendor/nanosvg/nanosvg.h
CFLAGS = --sysroot=$(SYSROOT) -O2 -fno-omit-frame-pointer -funwind-tables -Wall -Wextra -Werror \
         -Inative -Ivendor/nanosvg -D_GNU_SOURCE
# Not -lAcbAPI or -lcurl: their SONAMEs move between firmware releases (ACB is gone at webOS 5),
# so native/starfish.c and sn_net dlopen them instead of carrying a DT_NEEDED that would kill the
# process at exec() on the sets without them.
LIBS = -lSDL2 -lSDL2_ttf -lGLESv2 -lluna-service2 -lglib-2.0 -lwayland-client -lplayerAPIs -lpf-1.0 \
       -ldl -lrt -lpthread -lm
# libplayerAPIs' callback thunk reaches this symbol through a preemptible JUMP_SLOT: export exactly
# the C interposer (native/starfish.c), nothing else.
SMP_CALLBACK_HOOK = _ZN17StarfishMediaAPIs20callbackFunctionHookEixPKc
LDFLAGS = -Wl,--build-id=sha1 -Wl,--gc-sections -Wl,--export-dynamic-symbol=$(SMP_CALLBACK_HOOK)

# ---- the bundled libraries ----------------------------------------------------------------------
FFMPEG_DEFAULT_DIR = vendor/ffmpeg-prefix/lib
BUNDLED_LIB_DIR ?= $(FFMPEG_DEFAULT_DIR)
FFMPEG_REQUIRED = libavutil-sn.so.61 libswresample-sn.so.7 libavcodec-sn.so.63 libavformat-sn.so.63
# The software audio decoder (native/adec.c -> ci/build-adec.sh): FLAC/TrueHD/LPCM/Opus/... -> PCM.
ADEC_SO = $(BUNDLED_LIB_DIR)/libsnadec.so
# swscale is the dev capture stream's only; a release ships without it.
FFMPEG_OPTIONAL = $(if $(RELEASE),,libswscale-sn.so.10)
# libass renders text and styled (SRT/ASS/SSA) subtitles; the player runs without it (no text
# subtitles). By default `make ipk` builds it with ci/build-libass.sh; LIBASS_SO=<path> takes a
# prebuilt one instead.
LIBASS_DEFAULT_SO = vendor/libass-prefix/lib/libass-sn.so.0
LIBASS_SO ?= $(LIBASS_DEFAULT_SO)

# ---- the payload --------------------------------------------------------------------------------
ICON_FLAVOR = $(if $(filter beta,$(FLAVOR)),debug,$(FLAVOR))
ICONDIR   = $(if $(filter stable,$(FLAVOR)),pkg,pkg/$(ICON_FLAVOR))
ICONS     = $(ICONDIR)/icon.png $(ICONDIR)/largeIcon.png
APP_FILES = $(ICONS) pkg/splash.png pkg/appfont.ttf pkg/appfont-bold.ttf pkg/appfont-cjk.ttf pkg/appfont-emoji.ttf pkg/OFL.txt \
            pkg/le-roots.pem LICENSE NOTICE
LICENSE_TEXTS = $(sort $(wildcard licenses/*.txt))
SUBS_FONT_FILES = $(sort $(wildcard pkg/fonts/subs/*.ttf)) pkg/fonts/subs/manifest.json
STAGE = $(BUILD)/stage/$(FLAVOR)/app
IPK   = $(BUILD)/$(APPID)_$(IPK_VERSION)_arm.ipk

.PHONY: all ipk check selftest icons setup-env release-guard ffmpeg libass adec bundled-libs clean help FORCE \
        print-appid print-version print-ipk

all: $(BUILD)/stremionative $(BUILD)/stremionative-storage

help:
	@sed -n '2,10p' Makefile | sed 's/^# \{0,1\}//'

print-appid: ; @echo $(APPID)
print-version: ; @echo $(IPK_VERSION)
print-ipk: ; @echo $(IPK)

ndk-present:
	@test -x $(TOOLPREFIX)gcc || { echo "no webOS NDK at $(WEBOS_SDK): run \`make setup-env\` or pass WEBOS_SDK=<sdk root>"; exit 1; }
.PHONY: ndk-present

# ---- the NDK ------------------------------------------------------------------------------------
NDK_OS   := $(shell uname -s | tr A-Z a-z)
NDK_HOST := $(shell uname -m)
ifeq ($(NDK_OS),darwin)
  NDK_PLAT = darwin-$(if $(filter arm64,$(NDK_HOST)),arm64,x86_64)
else ifeq ($(NDK_HOST),aarch64)
  NDK_PLAT = linux-aarch64
else
  NDK_PLAT = UNSUPPORTED
endif
NDK_DL = $(dir $(WEBOS_SDK))ndk-download

setup-env:
	@test "$(NDK_PLAT)" != UNSUPPORTED || { \
	  echo "no webOS NDK is published for $(NDK_OS)-$(NDK_HOST). Available: darwin-arm64, darwin-x86_64, linux-aarch64."; exit 1; }
	@if test -x $(TOOLPREFIX)gcc; then echo "NDK already present at $(WEBOS_SDK)"; else \
	  set -e; mkdir -p $(dir $(WEBOS_SDK)); \
	  tarball=$$(python3 ci/deps.py fetch webos-ndk --asset $(NDK_PLAT) --into $(NDK_DL)); \
	  tar xjf "$$tarball" -C $(dir $(WEBOS_SDK)); \
	  cd $(WEBOS_SDK) && ./relocate-sdk.sh; rm -rf $(NDK_DL); \
	  echo "NDK ready: $$($(TOOLPREFIX)gcc --version | head -1)"; fi

# ---- C objects and the link ---------------------------------------------------------------------
$(NANOSVG):
	python3 ci/deps.py stage nanosvg --into vendor/nanosvg

$(BUILD)/obj/%.o: native/%.c $(wildcard native/*.h) $(NANOSVG) Makefile | ndk-present
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -c $< -o $@

# cargo decides whether anything changed (the sources, the features, SN_VERSION): it is always
# asked, and leaves the archive alone when nothing did.
$(RUST_LIB): FORCE | ndk-present
	$(RUST_VERSION_ENV) cargo +$(RUST_NIGHTLY) rustc --manifest-path crates/app/Cargo.toml --lib --crate-type staticlib \
	  --target $(RUST_TARGET) $(BUILD_STD) $(ARM_PROFILE_FLAG) $(RUST_FEATURES) --target-dir $(RUST_TDIR)

$(BUILD)/stremionative: $(OBJS) $(RUST_LIB) Makefile
	@mkdir -p $(BUILD)
	$(CC) $(CFLAGS) $(LDFLAGS) $(OBJS) $(RUST_LIB) $(LIBS) -o $@
	python3 ci/check-link-evidence.py $@

$(BUILD)/stremionative-storage: FORCE | ndk-present
	@mkdir -p $(BUILD)
	CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABI_LINKER=$(CURDIR)/ci/arm-cc.py \
	  cargo +$(RUST_NIGHTLY) rustc --manifest-path crates/storage/Cargo.toml --bin stremionative-storage \
	  --target $(RUST_TARGET) $(BUILD_STD) $(ARM_PROFILE_FLAG) --target-dir $(STORAGE_TDIR) -- \
	  -C link-arg=--sysroot=$(SYSROOT) -L native=$(SYSROOT)/usr/lib \
	  -C link-arg=-Wl,-rpath-link,$(SYSROOT)/usr/lib -C link-arg=-Wl,--build-id=sha1
	cp $(STORAGE_BIN) $@.tmp && chmod 755 $@.tmp && mv $@.tmp $@

# ---- FFmpeg -------------------------------------------------------------------------------------
# ci/build-ffmpeg.sh is incremental (it rebuilds only when its version, flags or RELEASE changed).
ffmpeg: | ndk-present
	RELEASE=$(RELEASE) WEBOS_SDK=$(WEBOS_SDK) sh ci/build-ffmpeg.sh

# ---- libass -------------------------------------------------------------------------------------
# One pinned ASS renderer and font stack on every firmware: libass, FreeType, FriBidi, HarfBuzz and
# libunibreak are built static (ci/dependencies.json, group "libass") and linked into ONE shared
# library with native/ass.c + native/ass_fonts.c, exporting only sn_ass_* (crates/media/src/player/
# ass.rs dlopens it by its SONAME from the app directory). Nothing binds to a firmware copy, and it
# needs only libc/libm/libgcc_s. The script is incremental (it rebuilds the stack only when a pin,
# a patch, the toolchain or the recipe changed; the facade is always relinked).
LIBASS_INPUTS = ci/build-libass.sh ci/build-libass.py ci/deps.py ci/dependencies.json ci/arm-cc.py \
                ci/check-link-evidence.py ci/patches/libass-0001-grow-font-faces.patch \
                native/ass.c native/ass_fonts.c native/ass_internal.h native/ass_composite.h native/ass.h
$(LIBASS_DEFAULT_SO): $(LIBASS_INPUTS) | ndk-present
	WEBOS_SDK=$(WEBOS_SDK) sh ci/build-libass.sh
libass: $(LIBASS_DEFAULT_SO)

# ---- the software audio decoder -----------------------------------------------------------------
# native/adec.c against the bundled FFmpeg's headers, linked to its -sn libraries by SONAME
# (crates/media/src/adec.rs dlopens it). ci/build-adec.sh fails on any DT_NEEDED beyond the -sn
# FFmpeg and libc/libm, and on any export outside sn_adec_*.
$(FFMPEG_DEFAULT_DIR)/libsnadec.so: native/adec.c ci/build-adec.sh | ndk-present ffmpeg
	WEBOS_SDK=$(WEBOS_SDK) sh ci/build-adec.sh
adec: $(FFMPEG_DEFAULT_DIR)/libsnadec.so

# The libraries the package needs, from BUNDLED_LIB_DIR; building them first when that is the
# default prefix. A missing required one stops here, by name.
bundled-libs: $(if $(filter $(FFMPEG_DEFAULT_DIR),$(BUNDLED_LIB_DIR)),ffmpeg adec,) $(if $(filter $(LIBASS_DEFAULT_SO),$(LIBASS_SO)),libass,)
	@missing=""; for l in $(FFMPEG_REQUIRED) libsnadec.so; do test -f "$(BUNDLED_LIB_DIR)/$$l" || missing="$$missing $$l"; done; \
	 if [ -n "$$missing" ]; then \
	   echo "error: the bundled FFmpeg is missing from $(BUNDLED_LIB_DIR):$$missing"; \
	   echo "       build it (make ffmpeg, ci/build-ffmpeg.sh) or pass BUNDLED_LIB_DIR=<dir with the -sn libraries>"; \
	   exit 1; fi
	@test -f "$(LIBASS_SO)" || echo "warning: no libass at $(LIBASS_SO) — the package shows no text subtitles (LIBASS_SO=<path>)"

# ---- the package --------------------------------------------------------------------------------
release-guard:
	@if [ "$(FLAVOR)" != debug ] && [ "$(RELEASE)" != 1 ]; then \
	  echo "error: FLAVOR=$(FLAVOR) is what other people install and is always a release build: add RELEASE=1"; exit 1; fi

ipk: release-guard bundled-libs $(BUILD)/stremionative $(BUILD)/stremionative-storage
	@echo "packaging $(if $(RELEASE),RELEASE,dev) ($(ARM_PROFILE)) as $(APPID) $(IPK_VERSION) [$(FLAVOR)]"
	rm -rf $(BUILD)/stage/$(FLAVOR) && mkdir -p $(STAGE)/fonts/subs
	cp $(BUILD)/stremionative $(APP_FILES) $(STAGE)/
	cp $(SUBS_FONT_FILES) $(STAGE)/fonts/subs/
	@# The bundled libraries' licence texts travel with them (LGPL-2.1 §6 for FriBidi): NOTICE.
	mkdir -p $(STAGE)/licenses && cp $(LICENSE_TEXTS) $(STAGE)/licenses/
	cp -L $(addprefix $(BUNDLED_LIB_DIR)/,$(FFMPEG_REQUIRED)) $(ADEC_SO) $(STAGE)/
	@for l in $(FFMPEG_OPTIONAL); do if [ -f "$(BUNDLED_LIB_DIR)/$$l" ]; then cp -L "$(BUNDLED_LIB_DIR)/$$l" $(STAGE)/; fi; done
	@if [ -f "$(LIBASS_SO)" ]; then cp -L "$(LIBASS_SO)" $(STAGE)/; fi
	@# Strip the STAGED copies only: build/stremionative keeps its symbols for crash triage.
	$(TOOLPREFIX)strip --strip-unneeded $(STAGE)/stremionative
	@for f in $(STAGE)/lib*-sn.so.* $(STAGE)/libsnadec.so; do $(TOOLPREFIX)strip --strip-unneeded "$$f" || exit 1; done
	mkdir -p $(BUILD)/stage/$(FLAVOR)/svc
	cp $(BUILD)/stremionative-storage $(BUILD)/stage/$(FLAVOR)/svc/stremionative-storage
	$(TOOLPREFIX)strip --strip-unneeded $(BUILD)/stage/$(FLAVOR)/svc/stremionative-storage
	@# Exactly one package in build/: release.yml publishes `ls build/*.ipk`.
	rm -f $(BUILD)/*.ipk
	SN_BETA_DATE=$(SN_BETA_DATE) python3 ci/mkipk.py --flavor $(FLAVOR) --payload $(STAGE) \
	  --storage $(BUILD)/stage/$(FLAVOR)/svc/stremionative-storage --out $(BUILD)
	@test -f $(IPK) || { echo "error: ci/mkipk.py did not write $(IPK)"; exit 1; }
	SN_BETA_DATE=$(SN_BETA_DATE) python3 ci/mkipk.py --verify $(FLAVOR) $(IPK)

# ---- host checks (no NDK) -----------------------------------------------------------------------
selftest:
	python3 -I ci/flavor.py --selftest
	python3 -I ci/mkipk.py --selftest
	python3 -I ci/hbrepo.py --selftest

check: selftest
	NO_PROXY="$$NO_PROXY,.invalid" no_proxy="$$no_proxy,.invalid" SNATIVE_RUNTIME_DIR=$$(mktemp -d) \
	  CARGO_INCREMENTAL=0 cargo test --workspace --no-fail-fast

icons:
	python3 ci/make-icons.py

clean:
	rm -rf $(BUILD) target/arm-app target/arm-storage

FORCE:
