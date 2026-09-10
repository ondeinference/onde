#!/usr/bin/env python3
"""Ask crates.io whether a version of a crate is already published.

Exits 0 if it is, 1 if it is not, and 2 if the question could not be answered.
Release workflows use the exit code to decide whether to skip a publish, and
again afterwards to assert the publish actually landed.

This exists because the obvious answer is wrong. `cargo info onde@1.2.4` run
from inside the onde workspace resolves `onde` against the local package, not
the registry, so it succeeds for whatever version the working tree happens to
carry. The release workflow used that as its idempotency guard, concluded every
version was already published, skipped the upload, and exited green: 1.2.3 and
1.2.4 were tagged and never reached crates.io. Query the index directly and
keep cargo out of it.

Usage:
    crates-io-published.py 1.2.5            # defaults to the `onde` crate
    crates-io-published.py 1.2.5 --crate x
    crates-io-published.py 1.2.5 --wait     # retry until it appears
"""

import argparse
import json
import sys
import time
import urllib.error
import urllib.request

INDEX = "https://index.crates.io"
USER_AGENT = "onde-release (https://github.com/ondeinference/onde)"


def index_path(crate):
    """The sparse index shards by name length, then by leading characters."""
    name = crate.lower()
    if len(name) == 1:
        return "1/{}".format(name)
    if len(name) == 2:
        return "2/{}".format(name)
    if len(name) == 3:
        return "3/{}/{}".format(name[0], name)
    return "{}/{}/{}".format(name[0:2], name[2:4], name)


def published_versions(crate):
    url = "{}/{}".format(INDEX, index_path(crate))
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            body = response.read().decode("utf-8")
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return []
        raise
    versions = []
    for line in body.splitlines():
        line = line.strip()
        if line:
            versions.append(json.loads(line)["vers"])
    return versions


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("version")
    parser.add_argument("--crate", default="onde")
    parser.add_argument(
        "--wait",
        action="store_true",
        help="retry until the version appears; the index is a CDN and lags a publish",
    )
    parser.add_argument("--attempts", type=int, default=10)
    parser.add_argument("--interval", type=int, default=15)
    args = parser.parse_args()

    attempts = args.attempts if args.wait else 1
    for attempt in range(1, attempts + 1):
        try:
            versions = published_versions(args.crate)
        except (urllib.error.URLError, OSError, ValueError) as error:
            print("could not reach the crates.io index: {}".format(error), file=sys.stderr)
            return 2
        if args.version in versions:
            print("{} {} is on crates.io".format(args.crate, args.version))
            return 0
        if attempt < attempts:
            print(
                "attempt {}: {} {} not visible yet, waiting {}s".format(
                    attempt, args.crate, args.version, args.interval
                )
            )
            time.sleep(args.interval)

    print("{} {} is not on crates.io".format(args.crate, args.version))
    return 1


if __name__ == "__main__":
    sys.exit(main())
