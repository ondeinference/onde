.PHONY: help patch minor major custom release release-commit tag sync-version relock verify lint-publish

SYNC := python3 scripts/sync-release-version.py
VERSION = $(shell $(SYNC) --print)

# Branch the tag must live on. Unlike smbcloud-cli (which tags development),
# onde requires the tag to point at a commit on main that carries every
# workflow file — see .agents/skills/sdk-releases/SKILL.md.
RELEASE_BRANCH := main

help:
	@echo "Release commands:"
	@echo "  make patch                  # 1.2.2 -> 1.2.3"
	@echo "  make minor                  # 1.2.2 -> 1.3.0"
	@echo "  make major                  # 1.2.2 -> 2.0.0"
	@echo "  make custom VERSION=1.3.0"
	@echo ""
	@echo "  ...then fill in the CHANGELOG stubs and:"
	@echo "  make release-commit         # verify + commit the bump"
	@echo "  make tag                    # on $(RELEASE_BRANCH), after merging"
	@echo ""
	@echo "Utilities:"
	@echo "  make verify                 # check every version source agrees"
	@echo "  make lint-publish           # check Cargo.toml has no git deps"
	@echo "  make sync-version           # re-propagate the current version"
	@echo "  make relock                 # re-resolve lockfiles (only when deps change)"

release:
	@test -n "$(BUMP)" || (echo "BUMP is required" && exit 1)
	@if [ "$(BUMP)" = "custom" ] && [ -z "$(VERSION_ARG)" ]; then \
		echo "VERSION is required for custom releases"; exit 1; \
	fi
	@case "$$(git branch --show-current)" in \
		development|release/*) ;; \
		*) echo "Releases must be prepared on development or release/*"; exit 1;; \
	esac
	@if [ -n "$$(git status --short --untracked-files=all)" ]; then \
		echo "Working tree must be clean before preparing a release"; exit 1; \
	fi
	@if [ "$(BUMP)" = "custom" ]; then \
		$(SYNC) --set "$(VERSION_ARG)"; \
	else \
		$(SYNC) --bump "$(BUMP)"; \
	fi
	@echo ""
	@echo "Version bumped to $$($(SYNC) --print)."
	@echo "Next: describe the release in the CHANGELOG stubs, then 'make release-commit'."

patch:
	@$(MAKE) --no-print-directory release BUMP=patch

minor:
	@$(MAKE) --no-print-directory release BUMP=minor

major:
	@$(MAKE) --no-print-directory release BUMP=major

custom:
	@$(MAKE) --no-print-directory release BUMP=custom VERSION_ARG="$(VERSION)"

release-commit: verify
	@release_version="$$($(SYNC) --print)"; \
	git add -A; \
	git commit -m "$$release_version"; \
	echo ""; \
	echo "Committed $$release_version."; \
	echo "Next: git checkout $(RELEASE_BRANCH) && git merge development --no-ff --no-edit && make tag"

tag: verify
	@current="$$(git branch --show-current)"; \
	if [ "$$current" != "$(RELEASE_BRANCH)" ]; then \
		echo "Tag must be cut on $(RELEASE_BRANCH), currently on $$current"; exit 1; \
	fi
	@if [ -n "$$(git status --short --untracked-files=all)" ]; then \
		echo "Working tree must be clean before tagging"; exit 1; \
	fi
	@release_version="$$($(SYNC) --print)"; \
	if git rev-parse -q --verify "refs/tags/$$release_version" >/dev/null; then \
		echo "Tag $$release_version already exists — registries are immutable, bump instead"; \
		exit 1; \
	fi; \
	git tag "$$release_version"; \
	echo "Tagged $$release_version (bare semver — a 'v' prefix will not trigger CI)."; \
	echo "Next: git push <remote> development $(RELEASE_BRANCH) $$release_version"

sync-version:
	@$(SYNC)

verify:
	@$(SYNC) --check

lint-publish:
	@$(SYNC) --lint-publish

# A version bump patches the lockfile version strings in-process, so it needs
# no cargo and no network. Use this only when dependencies actually changed:
# cargo metadata re-resolves (no compile, unlike `cargo check`), but each
# manifest costs ~72s of network because the [patch.crates-io] sysctl git
# dependency is refetched on every resolve.
relock:
	@echo "Re-resolving lockfiles (network-bound, ~3min)..."
	@cargo metadata --format-version 1 --quiet >/dev/null
	@cargo metadata --format-version 1 --quiet \
		--manifest-path sdk/dart/rust/Cargo.toml >/dev/null
	@cargo metadata --format-version 1 --quiet \
		--manifest-path sdk/react-native/rust/Cargo.toml >/dev/null
	@echo "  Cargo.lock, sdk/dart/rust/Cargo.lock, sdk/react-native/rust/Cargo.lock"
