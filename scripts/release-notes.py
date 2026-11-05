#!/usr/bin/env python3
"""Render the GitHub Release body for an Onde release, one section per SDK.

Both release workflows (Kotlin and Swift) create the same GitHub Release, so
both call this script and get an identical body no matter which finishes last.
The body names the product and its platforms, gives each SDK an install line
and its own changes, and links the SDK page on ondeinference.com.

Where each SDK's text comes from:
    Rust          CHANGELOG.md, used as written
    Flutter       sdk/dart/CHANGELOG.md
    React Native  sdk/react-native/CHANGELOG.md
    Swift         commit subjects touching the Swift paths below (no changelog)
    Kotlin        commit subjects touching the Kotlin paths below (no changelog)

The range is (previous release, tag]. The previous release is the newest semver
release older than the tag, so versions that were tagged but never published
(1.3.3 and 1.3.4 before 1.3.5) are folded into the next release.

Usage:
    release-notes.py 1.3.5                       # previous release via gh
    release-notes.py 1.3.5 --previous 1.3.2      # explicit range start
    release-notes.py 1.3.5 --output dist/release-notes.md
"""

import argparse
import importlib.util
import json
import re
import subprocess
import sys
from pathlib import Path

_SYNC_PATH = Path(__file__).resolve().parent / "sync-release-version.py"
_spec = importlib.util.spec_from_file_location("sync_release_version", _SYNC_PATH)
_sync = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_sync)

REPO_ROOT = _sync.REPO_ROOT
SEMVER = _sync.SEMVER
ROOT_CHANGELOG = _sync.ROOT_CHANGELOG
DART_CHANGELOG = _sync.DART_CHANGELOG
RN_CHANGELOG = _sync.RN_CHANGELOG

REPO = "ondeinference/onde"
REPO_URL = "https://github.com/" + REPO
SDK_URL = "https://ondeinference.com/sdk"

NO_CHANGES = "No changes in this release."
NO_ENTRY = "No changelog entry. See the pull requests below."
NO_ENTRY_BARE = "No changelog entry."

INTRO = (
    "Onde Inference runs LLMs on-device on iOS, macOS, tvOS, Android, Windows and Linux.\n"
    "One Rust core, with SDKs for Swift, Kotlin, Flutter, React Native and Rust."
)
FOOTER = (
    "Install guides and API reference: "
    "[Onde Inference SDKs for Swift, Kotlin, Flutter, React Native and Rust]({})".format(SDK_URL)
)

SWIFT_SCRIPT = ".github/scripts/build-swift-xcframework.sh"
SWIFT_PATHS = (
    SWIFT_SCRIPT,
    ".github/workflows/release-sdk-swift.yml",
    "src/inference/ffi.rs",
    "uniffi.toml",
)
KOTLIN_PATHS = (
    "sdk/kotlin/",
    ".github/workflows/release-sdk-kotlin.yml",
    "src/inference/ffi.rs",
    "uniffi.toml",
)
KOTLIN_EXCLUDES = ("sdk/kotlin/gradle.properties",)

# The Dart and React Native changelogs open every release with this sentence.
BOILERPLATE = re.compile(r"Version aligned with the Rust core\.\s*No [\w ]+? API changes\.\s*")

# "1.3.5", "Bump version to 1.3.5", "Release 1.3.5": commits that only bump.
VERSION_BUMP = re.compile(r"^(?:(?:bump|release|version)\b[^\n]*?)?v?\d+\.\d+\.\d+$", re.IGNORECASE)

SECTION = re.compile(r"^##\s+(\S+)\s*$", re.MULTILINE)


class NotesError(Exception):
    """The release notes could not be assembled."""


def semver_key(version):
    return tuple(int(part) for part in version.split("."))


def run(args, check=True):
    result = subprocess.run(
        args, cwd=str(REPO_ROOT), capture_output=True, text=True, encoding="utf-8"
    )
    if check and result.returncode != 0:
        raise NotesError("{} failed: {}".format(" ".join(args[:3]), result.stderr.strip()))
    return result


def git(*args):
    return run(["git"] + list(args)).stdout


def ref_exists(ref):
    return run(["git", "rev-parse", "--verify", "--quiet", ref + "^{commit}"], check=False).returncode == 0


def path_exists(ref, path):
    return bool(git("ls-tree", ref, "--", path).strip())


def release_tags():
    """Published GitHub Release tags, falling back to git tags if gh fails."""
    result = run(["gh", "release", "list", "--limit", "200", "--json", "tagName"], check=False)
    if result.returncode == 0:
        names = [entry["tagName"] for entry in json.loads(result.stdout)]
    else:
        names = git("tag", "--list").split()
    return [name for name in names if SEMVER.match(name)]


def previous_release(tag):
    older = [t for t in release_tags() if semver_key(t) < semver_key(tag)]
    return max(older, key=semver_key) if older else None


def changelog_sections(path):
    """Map version -> section body for a `## X.Y.Z` changelog."""
    if not path.exists():
        return {}
    content = path.read_text(encoding="utf-8")
    matches = list(SECTION.finditer(content))
    sections = {}
    for index, match in enumerate(matches):
        end = matches[index + 1].start() if index + 1 < len(matches) else len(content)
        sections[match.group(1)] = content[match.end():end].strip()
    return sections


def versions_in_range(sections, previous, tag):
    low = semver_key(previous) if previous else (-1,)
    high = semver_key(tag)
    found = [v for v in sections if SEMVER.match(v) and low < semver_key(v) <= high]
    return sorted(found, key=semver_key, reverse=True)


def bullets(body):
    """Top-level bullets of a section, each joined onto one line."""
    items = []
    for line in body.splitlines():
        if re.match(r"^[-*]\s+", line):
            items.append(re.sub(r"^[-*]\s+", "", line).strip())
        elif items and line.startswith((" ", "\t")) and line.strip():
            items[-1] += " " + line.strip()
    return items


def render_versions(entries):
    """Render [(version, lines)] newest first, with sub-headings when folded."""
    entries = [(version, lines) for version, lines in entries if lines]
    if not entries:
        return None
    if len(entries) == 1:
        return "\n".join(entries[0][1])
    return "\n\n".join("#### {}\n\n{}".format(v, "\n".join(lines)) for v, lines in entries)


def sdk_changelog_text(path, previous, tag):
    """Bullets from a Dart or React Native changelog, boilerplate removed."""
    sections = changelog_sections(path)
    entries = []
    for version in versions_in_range(sections, previous, tag):
        lines = []
        for item in bullets(sections[version]):
            item = BOILERPLATE.sub("", item).strip()
            if item:
                lines.append("- " + item)
        entries.append((version, lines))
    return render_versions(entries) or NO_CHANGES


def root_changelog_text(previous, tag, has_prs):
    """Root changelog sections as written, with their headings pushed down."""
    sections = changelog_sections(ROOT_CHANGELOG)
    versions = versions_in_range(sections, previous, tag)
    if not versions:
        return NO_ENTRY if has_prs else NO_ENTRY_BARE
    sub = "####" if len(versions) == 1 else "#####"
    entries = []
    for version in versions:
        body = re.sub(r"^#{3,6}\s+", sub + " ", sections[version], flags=re.MULTILINE)
        entries.append((version, body.splitlines()))
    return render_versions(entries)


def commit_text(previous, tag, to_ref, paths, excludes=()):
    """Commit subjects in the range that touch an SDK's own paths."""
    spec = ["{}..{}".format(previous, to_ref)] if previous else [to_ref]
    pathspec = list(paths) + [":(exclude){}".format(p) for p in excludes]
    log = git("log", "--no-merges", "--format=%s", *spec, "--", *pathspec)
    lines = []
    for subject in log.splitlines():
        subject = subject.strip()
        if not subject or VERSION_BUMP.match(subject):
            continue
        subject = re.sub(r"\(#(\d+)\)", r"([#\1]({}/pull/\1))".format(REPO_URL), subject)
        line = "- " + subject
        if line not in lines:
            lines.append(line)
    return "\n".join(lines) or NO_CHANGES


def pull_requests(tag, previous):
    """The `What's Changed` list from GitHub, or None if it is unavailable."""
    if not previous:
        return None
    result = run(
        [
            "gh", "api", "repos/{}/releases/generate-notes".format(REPO),
            "-f", "tag_name=" + tag, "-f", "previous_tag_name=" + previous,
        ],
        check=False,
    )
    if result.returncode != 0:
        return None
    try:
        body = json.loads(result.stdout).get("body", "")
    except ValueError:
        return None
    items = [line for line in body.splitlines() if line.startswith("* ")]
    return "\n".join(items) or None


def render(tag, previous):
    to_ref = tag if ref_exists(tag) else "HEAD"

    def section(title, install, changelog, text):
        head = "### {}\n{}".format(title, install)
        if changelog:
            head += " · [Changelog]({}/blob/main/{})".format(REPO_URL, changelog)
        # A blank line keeps a leading sub-heading from fusing with the install line.
        separator = "\n\n" if text.startswith("#") else "\n"
        return head + separator + text

    parts = [INTRO]

    if path_exists(to_ref, SWIFT_SCRIPT):
        parts.append(section(
            "Swift (iOS, macOS, tvOS)",
            '`.package(url: "https://github.com/ondeinference/onde-swift", from: "{}")`'.format(tag),
            None, commit_text(previous, tag, to_ref, SWIFT_PATHS),
        ))
    if path_exists(to_ref, "sdk/kotlin"):
        parts.append(section(
            "Kotlin Multiplatform (Android, JVM)",
            '`implementation("com.ondeinference:onde-inference:{}")`'.format(tag),
            None, commit_text(previous, tag, to_ref, KOTLIN_PATHS, KOTLIN_EXCLUDES),
        ))
    if path_exists(to_ref, "sdk/dart"):
        parts.append(section(
            "Flutter", "`flutter pub add onde_inference`", "sdk/dart/CHANGELOG.md",
            sdk_changelog_text(DART_CHANGELOG, previous, tag),
        ))
    if path_exists(to_ref, "sdk/react-native"):
        parts.append(section(
            "React Native", "`npm install @ondeinference/react-native`",
            "sdk/react-native/CHANGELOG.md",
            sdk_changelog_text(RN_CHANGELOG, previous, tag),
        ))
    prs = pull_requests(tag, previous)
    parts.append(section("Rust", "`cargo add onde`", "CHANGELOG.md", root_changelog_text(previous, tag, bool(prs))))

    if prs:
        parts.append("### Pull requests\n" + prs)

    if previous:
        compare = "{}/compare/{}...{}".format(REPO_URL, previous, tag)
    else:
        compare = "{}/commits/{}".format(REPO_URL, tag)
    parts.append("**Full changelog**: " + compare)
    parts.append("---\n" + FOOTER)
    # Changelog prose predates the no-em-dash rule for release bodies.
    return "\n\n".join(parts).replace(" \u2014 ", ", ").replace("\u2014", ", ") + "\n"


def main():
    parser = argparse.ArgumentParser(description="Render GitHub Release notes for a tag.")
    parser.add_argument("tag", help="release tag, for example 1.3.5")
    parser.add_argument("--previous", help="start of the range; default is the previous GitHub Release")
    parser.add_argument("--output", help="write here instead of stdout")
    args = parser.parse_args()

    if not SEMVER.match(args.tag):
        print("error: tag must look like 1.2.3, got {}".format(args.tag), file=sys.stderr)
        return 1
    try:
        previous = args.previous or previous_release(args.tag)
        if previous and not ref_exists(previous):
            raise NotesError("previous tag {} is not in this checkout (fetch-depth: 0?)".format(previous))
        body = render(args.tag, previous)
    except NotesError as error:
        print("error: {}".format(error), file=sys.stderr)
        return 1

    if args.output:
        out = Path(args.output)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(body, encoding="utf-8")
    else:
        sys.stdout.write(body)
    return 0


if __name__ == "__main__":
    sys.exit(main())
