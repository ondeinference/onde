#!/usr/bin/env python3
"""Propagate the release version from Cargo.toml to every other version source.

The root Cargo.toml `[package] version` is the single source of truth. Every
other manifest in the repo is derived from it by this script, so a release bump
is one command instead of the ten-row manual checklist in
.agents/skills/sdk-releases/SKILL.md.

Every substitution fails loudly if its pattern stops matching, so a manifest
that gets restructured breaks the release instead of silently keeping a stale
version. That is how sdk/react-native/package-lock.json drifted four releases
behind (1.1.2 while package.json said 1.2.1) without anything noticing.

Usage:
    sync-release-version.py                  # re-propagate current version
    sync-release-version.py --bump patch     # 1.2.2 -> 1.2.3, then propagate
    sync-release-version.py --set 1.3.0      # set explicitly, then propagate
    sync-release-version.py --check          # verify only, no writes (CI gate)
    sync-release-version.py --print          # print current version, no writes
"""

import argparse
import json
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

CARGO_TOML = REPO_ROOT / "Cargo.toml"
DART_PUBSPEC = REPO_ROOT / "sdk/dart/pubspec.yaml"
DART_CHANGELOG = REPO_ROOT / "sdk/dart/CHANGELOG.md"
RN_PACKAGE_JSON = REPO_ROOT / "sdk/react-native/package.json"
RN_PACKAGE_LOCK = REPO_ROOT / "sdk/react-native/package-lock.json"
RN_RUST_CARGO = REPO_ROOT / "sdk/react-native/rust/Cargo.toml"
RN_CHANGELOG = REPO_ROOT / "sdk/react-native/CHANGELOG.md"
KOTLIN_PROPS = REPO_ROOT / "sdk/kotlin/gradle.properties"

CHANGELOGS = (DART_CHANGELOG, RN_CHANGELOG)

# Cargo.lock entries carrying a release version, and the packages to patch in
# each. A version bump only changes these strings, so they are rewritten
# directly instead of shelling out to cargo: `cargo metadata` re-resolves over
# the network (~72s per manifest here, because the [patch.crates-io] sysctl git
# dependency is refetched on every resolve), which made the bump both slow and
# dependent on the network. Run `make relock` when dependencies actually change.
LOCKFILES = (
    (REPO_ROOT / "Cargo.lock", ("onde",)),
    (REPO_ROOT / "sdk/dart/rust/Cargo.lock", ("onde",)),
    (REPO_ROOT / "sdk/react-native/rust/Cargo.lock", ("onde", "onde-react-native")),
)

# A freshly bumped changelog section carries this placeholder. `--check` refuses
# to pass while it is still there, so an unwritten changelog blocks the release
# rather than shipping stale notes to pub.dev and npm.
PLACEHOLDER = "_TODO: describe this release_"

SEMVER = re.compile(r"^\d+\.\d+\.\d+$")


class SyncError(Exception):
    """A version source could not be read or updated."""


def rel(path):
    return path.relative_to(REPO_ROOT).as_posix()


def read_text(path):
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        raise SyncError("missing file: {}".format(rel(path)))


def write_text(path, content):
    path.write_text(content, encoding="utf-8")


def cargo_package_version(path):
    """Read `version` from the [package] table, ignoring dependency versions."""
    section = None
    for line in read_text(path).splitlines():
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            section = stripped[1:-1]
            continue
        if section == "package":
            match = re.match(r'version\s*=\s*"([^"]+)"', stripped)
            if match:
                return match.group(1)
    raise SyncError("no [package] version in {}".format(rel(path)))


def set_cargo_package_version(path, version):
    """Rewrite only the [package] table's version line."""
    lines = read_text(path).splitlines(keepends=True)
    section = None
    for index, line in enumerate(lines):
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            section = stripped[1:-1]
            continue
        if section == "package" and re.match(r'version\s*=\s*"[^"]+"', stripped):
            lines[index] = re.sub(
                r'(version\s*=\s*")[^"]+(")',
                r"\g<1>{}\g<2>".format(version),
                line,
                count=1,
            )
            write_text(path, "".join(lines))
            return
    raise SyncError("no [package] version in {}".format(rel(path)))


def lock_version_pattern(name):
    """Match the version line of a [[package]] entry by package name."""
    return re.compile(
        r'^(name = "{}"\nversion = ")([^"]+)(")'.format(re.escape(name)), re.MULTILINE
    )


def read_lock_version(path, name):
    match = lock_version_pattern(name).search(read_text(path))
    if match is None:
        raise SyncError("no '{}' package entry in {}".format(name, rel(path)))
    return match.group(2)


def set_lock_version(path, name, version):
    content = read_text(path)
    updated, count = lock_version_pattern(name).subn(
        r"\g<1>{}\g<3>".format(version), content, count=1
    )
    if count == 0:
        raise SyncError("no '{}' package entry in {}".format(name, rel(path)))
    if updated != content:
        write_text(path, updated)
        return True
    return False


def sub_or_raise(content, pattern, replacement, path, description):
    updated, count = pattern.subn(replacement, content, count=1)
    if count == 0:
        raise SyncError(
            "could not update {} in {} — the file layout changed, "
            "fix the pattern in scripts/sync-release-version.py".format(
                description, rel(path)
            )
        )
    return updated


def update_json_versions(path, version, keys):
    """Rewrite named version fields, preserving npm's on-disk JSON formatting."""
    data = json.loads(read_text(path))
    for key_path in keys:
        node = data
        for key in key_path[:-1]:
            if key not in node:
                raise SyncError(
                    "missing key {} in {}".format(
                        ".".join(str(k) for k in key_path), rel(path)
                    )
                )
            node = node[key]
        node[key_path[-1]] = version
    write_text(path, json.dumps(data, indent=2, ensure_ascii=False) + "\n")


def propagate(version):
    """Write `version` into every derived manifest. Returns changed paths."""
    before = {
        path: read_text(path)
        for path in (
            DART_PUBSPEC,
            RN_PACKAGE_JSON,
            RN_PACKAGE_LOCK,
            RN_RUST_CARGO,
            KOTLIN_PROPS,
        )
    }

    write_text(
        DART_PUBSPEC,
        sub_or_raise(
            before[DART_PUBSPEC],
            re.compile(r"^version:.*$", re.MULTILINE),
            "version: {}".format(version),
            DART_PUBSPEC,
            "pubspec version",
        ),
    )

    write_text(
        KOTLIN_PROPS,
        sub_or_raise(
            before[KOTLIN_PROPS],
            re.compile(r"^VERSION_NAME=.*$", re.MULTILINE),
            "VERSION_NAME={}".format(version),
            KOTLIN_PROPS,
            "VERSION_NAME",
        ),
    )

    set_cargo_package_version(RN_RUST_CARGO, version)

    update_json_versions(RN_PACKAGE_JSON, version, [("version",)])
    # lockfileVersion 3 records the root package version twice.
    update_json_versions(
        RN_PACKAGE_LOCK, version, [("version",), ("packages", "", "version")]
    )

    changed = []
    for path, original in before.items():
        if read_text(path) != original:
            changed.append(path)

    for lock_path, names in LOCKFILES:
        # List, not a generator: any() short-circuits and would skip the
        # remaining packages in a lockfile that carries more than one.
        if any([set_lock_version(lock_path, name, version) for name in names]):
            changed.append(lock_path)

    return changed


def ensure_changelog_section(path, version):
    """Prepend a stub section for `version` if the changelog lacks one."""
    content = read_text(path)
    if re.search(r"^##\s+{}\s*$".format(re.escape(version)), content, re.MULTILINE):
        return False
    write_text(path, "## {}\n\n- {}\n\n{}".format(version, PLACEHOLDER, content))
    return True


def check(version):
    """Verify every version source agrees with Cargo.toml. Returns problems."""
    problems = []

    def expect(path, actual, label):
        if actual != version:
            problems.append(
                "{}: {} is {}, expected {}".format(rel(path), label, actual, version)
            )

    pubspec = re.search(r"^version:\s*(\S+)\s*$", read_text(DART_PUBSPEC), re.MULTILINE)
    if pubspec is None:
        problems.append("{}: no version line".format(rel(DART_PUBSPEC)))
    else:
        expect(DART_PUBSPEC, pubspec.group(1), "version")

    props = re.search(r"^VERSION_NAME=(.*)$", read_text(KOTLIN_PROPS), re.MULTILINE)
    if props is None:
        problems.append("{}: no VERSION_NAME".format(rel(KOTLIN_PROPS)))
    else:
        expect(KOTLIN_PROPS, props.group(1).strip(), "VERSION_NAME")

    expect(RN_RUST_CARGO, cargo_package_version(RN_RUST_CARGO), "[package] version")

    expect(
        RN_PACKAGE_JSON, json.loads(read_text(RN_PACKAGE_JSON)).get("version"), "version"
    )

    lock = json.loads(read_text(RN_PACKAGE_LOCK))
    expect(RN_PACKAGE_LOCK, lock.get("version"), "version")
    expect(
        RN_PACKAGE_LOCK,
        lock.get("packages", {}).get("", {}).get("version"),
        'packages[""].version',
    )

    for lock_path, names in LOCKFILES:
        for name in names:
            expect(lock_path, read_lock_version(lock_path, name), "{} version".format(name))

    for changelog in CHANGELOGS:
        content = read_text(changelog)
        section = re.search(
            r"^##\s+{}\s*$(.*?)(?=^##\s|\Z)".format(re.escape(version)),
            content,
            re.MULTILINE | re.DOTALL,
        )
        if section is None:
            problems.append("{}: no '## {}' section".format(rel(changelog), version))
        elif PLACEHOLDER in section.group(1):
            problems.append(
                "{}: '## {}' still has the {} placeholder".format(
                    rel(changelog), version, PLACEHOLDER
                )
            )

    return problems


def lint_publishable():
    """Find git dependencies, which `cargo publish` rejects outright.

    This is the 1.2.0 crates.io failure: the mistralrs deps were pointed at the
    fork's git branch, and the release burned 13 minutes of macOS runner time to
    surface an error the manifest alone proves in milliseconds. Cargo.toml
    documents pointing them back at git for local fork work, so this is a
    release gate rather than part of --check.

    [patch.crates-io] is exempt: crates.io consumers ignore it entirely.
    """
    problems = []
    section = None

    for number, line in enumerate(read_text(CARGO_TOML).splitlines(), start=1):
        stripped = line.strip()
        if stripped.startswith("#"):
            continue
        if stripped.startswith("[") and stripped.endswith("]"):
            section = stripped[1:-1]
            continue
        if section is None or section.startswith("patch."):
            continue

        is_dependency_table = (
            section in ("dependencies", "build-dependencies", "dev-dependencies")
            or section.startswith(
                ("dependencies.", "build-dependencies.", "dev-dependencies.")
            )
            or section.endswith(".dependencies")
        )
        if not is_dependency_table:
            continue

        if re.search(r"\bgit\s*=", stripped):
            problems.append(
                "Cargo.toml:{}: git dependency in [{}] — cargo publish "
                "rejects it: {}".format(number, section, stripped)
            )

    return problems


def bump(version, part):
    major, minor, patch = (int(piece) for piece in version.split("."))
    if part == "major":
        return "{}.0.0".format(major + 1)
    if part == "minor":
        return "{}.{}.0".format(major, minor + 1)
    return "{}.{}.{}".format(major, minor, patch + 1)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--bump", choices=("major", "minor", "patch"))
    group.add_argument("--set", dest="explicit", metavar="X.Y.Z")
    group.add_argument("--check", action="store_true")
    group.add_argument("--print", dest="print_only", action="store_true")
    group.add_argument("--lint-publish", dest="lint_publish", action="store_true")
    args = parser.parse_args()

    try:
        current = cargo_package_version(CARGO_TOML)

        if args.print_only:
            print(current)
            return 0

        if args.lint_publish:
            problems = lint_publishable()
            if problems:
                print("Cargo.toml is not publishable to crates.io:")
                for problem in problems:
                    print("  - {}".format(problem))
                return 1
            print("Cargo.toml is publishable: no git dependencies.")
            return 0

        if args.check:
            problems = check(current)
            if problems:
                print("Version sources disagree with Cargo.toml ({}):".format(current))
                for problem in problems:
                    print("  - {}".format(problem))
                return 1
            print("All version sources agree: {}".format(current))
            return 0

        version = current
        if args.explicit:
            if not SEMVER.match(args.explicit):
                raise SyncError(
                    "--set expects bare semver X.Y.Z, got '{}'".format(args.explicit)
                )
            version = args.explicit
        elif args.bump:
            version = bump(current, args.bump)

        if version != current:
            set_cargo_package_version(CARGO_TOML, version)
            print("Cargo.toml: {} -> {}".format(current, version))

        changed = propagate(version)
        stubbed = [c for c in CHANGELOGS if ensure_changelog_section(c, version)]

        print("Release version: {}".format(version))
        if changed:
            for path in changed:
                print("  updated {}".format(rel(path)))
        else:
            print("  all manifests already in sync")
        for path in stubbed:
            print("  added '## {}' stub to {} — fill it in".format(version, rel(path)))

        return 0
    except SyncError as error:
        print("error: {}".format(error), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
