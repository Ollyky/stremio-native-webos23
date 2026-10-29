#!/usr/bin/env python3
"""Generate a Homebrew Channel repository for StremioNative from a built .ipk.

Homebrew Channel (Settings > Add repository) takes the URL of a `repo.json` index. It fetches
the index, then each package's manifest, icon and .ipk, all anonymously over HTTPS. This writes
the files that URL serves:

    <out>/repo.json                        the index: one entry per package (merged into an
                                           existing repo.json in <out>, so stable and beta
                                           share one URL)
    <out>/<app id>.manifest.json           what the TV installs: version, ipk URL, sha256, size
    <out>/<ipk name>                       the package itself (copied unless --ipk-url is given)

Usage:
    python3 ci/hbrepo.py --ipk build/web.stremio.native.app_0.1.0_arm.ipk \\
        --base-url https://example.github.io/stremio-native --out _site

    python3 ci/hbrepo.py --selftest

`--base-url` is where `<out>` will be served. Every URL a TV follows is made absolute against it,
because Homebrew Channel resolves nothing relative. The version and id are read from the
package's own control file, so the manifest can never describe a different build than the file
it points at: Homebrew Channel decides an update exists by comparing the version string.
"""

import argparse
import hashlib
import io
import json
import os
import shutil
import sys
import tarfile
import tempfile

TITLE = "StremioNative"
SHORT_DESCRIPTION = "Native Stremio client for webOS: hardware playback, addons, live TV."
SOURCE_URL = "https://github.com/Morpheus235/stremio-native"


def ar_members(data: bytes) -> "dict[str, bytes]":
    """The members of an `ar` archive (an .ipk is one), by name."""
    if not data.startswith(b"!<arch>\n"):
        raise ValueError("not an ar archive")
    out, pos = {}, 8
    while pos + 60 <= len(data):
        header = data[pos:pos + 60]
        name = header[:16].decode().strip().rstrip("/")
        size = int(header[48:58].decode().strip())
        pos += 60
        out[name] = data[pos:pos + size]
        pos += size + (size & 1)
    return out


def control_of(ipk: bytes) -> "dict[str, str]":
    """The `control` file of an .ipk, as fields."""
    members = ar_members(ipk)
    name = next((n for n in members if n.startswith("control.tar")), None)
    if name is None:
        raise ValueError("ipk has no control archive")
    with tarfile.open(fileobj=io.BytesIO(members[name])) as tar:
        member = next((m for m in tar.getmembers() if m.name.lstrip("./") == "control"), None)
        if member is None:
            raise ValueError("control archive has no control file")
        text = tar.extractfile(member).read().decode()
    fields = {}
    for line in text.splitlines():
        if ":" in line and not line.startswith(" "):
            key, value = line.split(":", 1)
            fields[key.strip()] = value.strip()
    return fields


def build(ipk_path: str, base_url: str, ipk_url: "str | None", icon_url: str, title: str = TITLE) -> "dict[str, bytes]":
    """Every file the repository serves, as `{name: bytes}`."""
    with open(ipk_path, "rb") as f:
        ipk = f.read()
    control = control_of(ipk)
    app_id, version = control.get("Package"), control.get("Version")
    if not app_id or not version:
        raise ValueError(f"control file lacks Package/Version: {control}")
    base = base_url.rstrip("/")
    ipk_name = os.path.basename(ipk_path)
    manifest_name = f"{app_id}.manifest.json"
    manifest = {
        "id": app_id,
        "title": title,
        "version": version,
        "type": "native",
        "appDescription": SHORT_DESCRIPTION,
        "iconUri": icon_url,
        "sourceUrl": SOURCE_URL,
        "rootRequired": False,
        "ipkUrl": ipk_url or f"{base}/{ipk_name}",
        "ipkHash": {"sha256": hashlib.sha256(ipk).hexdigest()},
        "ipkSize": len(ipk),
    }
    index = {"packages": [{
        "id": app_id,
        "title": title,
        "shortDescription": SHORT_DESCRIPTION,
        "iconUri": icon_url,
        "manifestUrl": f"{base}/{manifest_name}",
    }]}
    files = {
        "repo.json": (json.dumps(index, indent=2) + "\n").encode(),
        manifest_name: (json.dumps(manifest, indent=2) + "\n").encode(),
    }
    if ipk_url is None:
        files[ipk_name] = ipk
    return files


def merge_index(existing: bytes, new: bytes) -> bytes:
    """Add `new`'s packages to the index `existing`, replacing any entry with the same app id, so
    one repo.json lists the stable app and the beta app side by side."""
    ours = json.loads(new)["packages"]
    ids = {p["id"] for p in ours}
    kept = [p for p in json.loads(existing).get("packages", []) if p.get("id") not in ids]
    return (json.dumps({"packages": kept + ours}, indent=2) + "\n").encode()


def main(argv=None) -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--ipk")
    p.add_argument("--base-url")
    p.add_argument("--out")
    p.add_argument("--ipk-url", help="serve the ipk from here (e.g. a release asset) instead of copying it")
    p.add_argument("--icon-url", help="defaults to <base-url>/icon160.png")
    p.add_argument("--title", default=TITLE, help="the listing title (e.g. \"StremioNative Beta\")")
    p.add_argument("--selftest", action="store_true")
    a = p.parse_args(argv)
    if a.selftest:
        return selftest()
    if not (a.ipk and a.base_url and a.out):
        p.error("--ipk, --base-url and --out are required")
    icon = a.icon_url or f"{a.base_url.rstrip('/')}/icon160.png"
    files = build(a.ipk, a.base_url, a.ipk_url, icon, a.title)
    os.makedirs(a.out, exist_ok=True)
    index_path = os.path.join(a.out, "repo.json")
    if os.path.exists(index_path):
        with open(index_path, "rb") as f:
            files["repo.json"] = merge_index(f.read(), files["repo.json"])
    for name, data in files.items():
        with open(os.path.join(a.out, name), "wb") as f:
            f.write(data)
        print(f"wrote {os.path.join(a.out, name)}")
    return 0


def _fake_ipk(app_id: str, version: str) -> bytes:
    control = f"Package: {app_id}\nVersion: {version}\nArchitecture: arm\n".encode()
    tar_buf = io.BytesIO()
    with tarfile.open(fileobj=tar_buf, mode="w:gz") as tar:
        info = tarfile.TarInfo("./control")
        info.size = len(control)
        tar.addfile(info, io.BytesIO(control))
    members = [("debian-binary", b"2.0\n"), ("control.tar.gz", tar_buf.getvalue()), ("data.tar.gz", b"")]
    out = bytearray(b"!<arch>\n")
    for name, data in members:
        out += f"{name + '/':<16}{0:<12}{0:<6}{0:<6}{'100644':<8}{len(data):<10}`\n".encode()
        out += data + (b"\n" if len(data) & 1 else b"")
    return bytes(out)


def selftest() -> int:
    tmp = tempfile.mkdtemp()
    try:
        ipk = _fake_ipk("web.stremio.native.app", "0.1.0")
        path = os.path.join(tmp, "web.stremio.native.app_0.1.0_arm.ipk")
        with open(path, "wb") as f:
            f.write(ipk)
        files = build(path, "https://host.example/repo/", None, "https://host.example/repo/icon160.png")
        index = json.loads(files["repo.json"])
        manifest = json.loads(files["web.stremio.native.app.manifest.json"])
        assert index["packages"][0]["manifestUrl"] == "https://host.example/repo/web.stremio.native.app.manifest.json"
        assert manifest["version"] == "0.1.0", manifest
        assert manifest["ipkUrl"] == "https://host.example/repo/web.stremio.native.app_0.1.0_arm.ipk"
        assert manifest["ipkHash"]["sha256"] == hashlib.sha256(ipk).hexdigest()
        assert manifest["ipkSize"] == len(ipk)
        assert files["web.stremio.native.app_0.1.0_arm.ipk"] == ipk
        files = build(path, "https://host.example/repo", "https://dl.example/x.ipk", "https://i.example/i.png")
        assert json.loads(files["web.stremio.native.app.manifest.json"])["ipkUrl"] == "https://dl.example/x.ipk"
        assert len(files) == 2, "an external ipk URL is not copied"
        files = build(path, "https://host.example/repo/beta", "https://dl.example/x.ipk", "https://i.example/i.png", "StremioNative Beta")
        assert json.loads(files["repo.json"])["packages"][0]["title"] == "StremioNative Beta"
        beta_ipk = _fake_ipk("web.stremio.native.app.beta", "0.2.20261016")
        beta_path = os.path.join(tmp, "beta.ipk")
        with open(beta_path, "wb") as f:
            f.write(beta_ipk)
        out = os.path.join(tmp, "site")
        os.makedirs(out)
        with open(os.path.join(out, "repo.json"), "w") as f:
            f.write('{"packages": []}')
        for ipk_path, title in [(path, "StremioNative"), (beta_path, "StremioNative Beta"), (beta_path, "StremioNative Beta")]:
            assert main(["--ipk", ipk_path, "--base-url", "https://h.example/r", "--ipk-url", "https://dl.example/x.ipk",
                         "--title", title, "--out", out]) == 0
        with open(os.path.join(out, "repo.json")) as f:
            merged = json.load(f)["packages"]
        assert [(p["id"], p["title"]) for p in merged] == [
            ("web.stremio.native.app", "StremioNative"),
            ("web.stremio.native.app.beta", "StremioNative Beta"),
        ], merged
        assert os.path.exists(os.path.join(out, "web.stremio.native.app.beta.manifest.json"))
        try:
            control_of(b"not an archive")
            raise AssertionError("garbage accepted")
        except ValueError:
            pass
    finally:
        shutil.rmtree(tmp)
    print("hbrepo selftest: ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
