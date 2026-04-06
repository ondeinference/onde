---
name: legal-and-trademarks
description: Legal compliance, license management, trademark guidelines, and distribution requirements for the Onde Inference project and its SDKs.
---

# Legal & Trademarks — Onde Inference

> Reference for AI agents and human contributors on all legal, licensing, trademark, and distribution compliance topics for the `onde` crate, `onde-swift`, and all downstream SDKs.

---

## License

Onde is dual-licensed under **MIT** and **Apache 2.0**, at the user's option. This mirrors the licensing model used by the Rust language itself.

```
SPDX-License-Identifier: MIT OR Apache-2.0
```

### Why dual license?

| Concern | MIT alone | MIT OR Apache-2.0 |
|---|---|---|
| Patent grant | ❌ None | ✅ Apache 2.0 includes irrevocable patent grant |
| Enterprise adoption | ⚠️ Legal teams may ask questions | ✅ Widely understood and accepted |
| Rust ecosystem compatibility | ✅ | ✅ |
| Contributor protection | ❌ | ✅ Patent termination clause deters trolling |
| Simplicity | ✅ | ✅ User just picks one |

### License files

| File | Contents |
|---|---|
| `LICENSE-MIT` | MIT License, copyright Onde Inference (Splitfire AB) |
| `LICENSE-APACHE` | Apache License 2.0 |

Never use a single `LICENSE` file for dual-licensed projects. Always maintain both files separately.

### Cargo.toml declaration

```toml
license = "MIT OR Apache-2.0"
```

Use `OR` (uppercase, SPDX standard), not `AND` or `/`. `AND` would mean the user must comply with **both** simultaneously — that is a different and more restrictive claim.

---

## Copyright

The canonical copyright notice for all Onde Inference files is:

```
Copyright (c) 2026 Onde Inference (Splitfire AB)
```

- **Onde Inference** — the product/brand name
- **Splitfire AB** — the Swedish legal entity (org. nr. on file)
- Use the year of first publication; do not update the year annually in LICENSE files

This notice must appear in:
- `LICENSE-MIT`
- `LICENSE-APACHE` (NOTICE file section)
- Any generated SDK packages distributed to registries (crates.io, pub.dev, npm, PyPI)

---

## Dependency Licenses

### Direct dependencies — audit table

| Crate | License | Notes |
|---|---|---|
| `mistral.rs` (Eric Buehler) | MIT | Must preserve `Copyright (c) 2024 Eric Buehler` |
| `uniffi` | MPL-2.0 | File-level copyleft; compatible with MIT/Apache 2.0 for binary distribution |
| `tokio` | MIT | — |
| `serde` | MIT OR Apache-2.0 | — |
| `anyhow` / `thiserror` | MIT OR Apache-2.0 | — |
| `hf-hub` | Apache-2.0 | — |
| `log` | MIT OR Apache-2.0 | — |

### MPL-2.0 (`uniffi`) — what it means in practice

Mozilla Public License 2.0 is **file-level copyleft**:
- You may distribute binaries combining MPL code with MIT/Apache code without restriction.
- If you **modify** MPL-licensed source files, you must make those modifications available.
- As a consumer of `uniffi` (not modifying its internals), **no disclosure obligation applies**.

### GPL — hard blocker

Never introduce a GPL-licensed dependency (v2-only in particular). GPL v2 is **incompatible** with Apache 2.0 and would contaminate the entire binary.

Before adding any new dependency, verify its license with:

```bash
cargo license --all-features
```

Acceptable licenses for new dependencies: MIT, Apache-2.0, MIT OR Apache-2.0, ISC, BSD-2-Clause, BSD-3-Clause, MPL-2.0, Unicode-3.0, Zlib.

---

## Model Licenses

**This is the most critical compliance area for downstream users.**

Onde downloads AI models from HuggingFace at runtime. These models carry their own licenses, entirely separate from Onde's MIT OR Apache-2.0 license. Any application that ships with or downloads these models must comply with the model's license.

### Current model license table

| Model | HuggingFace Repo | License | Commercial use |
|---|---|---|---|
| Qwen 2.5 1.5B Instruct | `Qwen/Qwen2.5-1.5B-Instruct` | [Qwen Community License](https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct/blob/main/LICENSE) | ✅ Allowed with conditions |
| Qwen 2.5 3B Instruct | `Qwen/Qwen2.5-3B-Instruct` | Qwen Community License | ✅ Allowed with conditions |
| Qwen 2.5 Coder 1.5B | `Qwen/Qwen2.5-Coder-1.5B-Instruct` | Qwen Community License | ✅ Allowed with conditions |
| Qwen 2.5 Coder 3B | `Qwen/Qwen2.5-Coder-3B-Instruct` | Qwen Community License | ✅ Allowed with conditions |
| Qwen 2.5 Coder 7B | `Qwen/Qwen2.5-Coder-7B-Instruct` | Qwen Community License | ✅ Allowed with conditions |

### Key Qwen Community License restrictions

1. **No training competing models** — you may not use outputs or weights to train a model that competes with Qwen/Alibaba offerings.
2. **Attribution required** — downstream products must acknowledge the use of Qwen models.
3. **No misrepresentation** — you may not claim the model is your own original work.
4. **Threshold clause** — organisations with >100 million monthly active users must obtain a separate commercial licence from Alibaba Cloud.

### Disclosure obligation

Onde's README and any SDK README must contain a `## Model Licenses` section that:
- Lists every bundled or downloadable model
- Links to the model's license on HuggingFace
- States that the model license is independent of Onde's own license

When a new model is added to `src/inference/models.rs`, the model license table in the README **must be updated in the same commit**.

---

## Trademark Guidelines

### Onde Inference brand

**"Onde Inference"** and the Onde logo (`assets/onde-inference-logo.svg`) are trademarks of Splitfire AB.

MIT and Apache 2.0 grant code rights only — **neither license grants trademark rights**. This is intentional and protective.

#### Permitted uses

- Referring to the project by name: *"built with Onde Inference"*, *"powered by Onde"*
- Linking to the official website or repository
- Stating compatibility: *"compatible with Onde Inference 0.x"*
- Press, editorial, and factual references

#### Prohibited uses

- Using "Onde Inference" or the logo as the name of a fork, competing product, or derived SDK
- Implying official endorsement or affiliation without written permission from Splitfire AB
- Using the logo in a way that could cause confusion about the origin of a product

#### Forks

A fork of `onde` may use the code freely under MIT OR Apache-2.0 but **must not**:
- Use the "Onde Inference" name as its product name
- Use the Onde logo
- Claim to be the official Onde Inference project

This is standard open-source brand protection and is consistent with how the Apache Software Foundation, Mozilla, and Rust Foundation handle their trademarks.

---

## mistral.rs Fork — Attribution Requirements

Onde depends on a personal fork of `mistral.rs` by Eric Buehler:

```toml
mistralrs = { git = "https://github.com/setoelkahfi/mistral.rs",
              branch = "fix/all-platform-fixes", ... }
```

### Obligations

1. **Preserve the upstream LICENSE** — `mistral.rs/LICENSE` must remain intact and contain `Copyright (c) 2024 Eric Buehler`. Never alter it.
2. **Pin to a commit SHA, not a branch** — branches can be force-pushed or deleted. For any production release, the dependency must be pinned:
   ```toml
   mistralrs = { git = "https://github.com/setoelkahfi/mistral.rs", rev = "<commit-sha>" }
   ```
3. **Upstream-first policy** — fixes that are not Onde-specific should be submitted as PRs to `EricLBuehler/mistral.rs`. Maintaining a long-lived divergent fork creates legal ambiguity about authorship of patches.
4. **NOTICE file** — if Onde ever produces an Apache 2.0 binary distribution (e.g., via PyPI), a `NOTICE` file must include: *"This product includes software developed by Eric Buehler (mistral.rs)."*

---

## Distribution Registry Compliance

### crates.io

- `license = "MIT OR Apache-2.0"` in `Cargo.toml` ✅
- Both `LICENSE-MIT` and `LICENSE-APACHE` must be present in the repo root
- `readme = "README.md"` must point to a file that contains the `## Model Licenses` section

### pub.dev (Flutter/Dart)

- `pubspec.yaml` must declare `license: MIT OR Apache-2.0`
- The `## Model Licenses` section must be present in the published README
- pub.dev displays the license on the package page — verify after each publish

### Swift Package Manager / onde-swift

- `onde-swift/README.md` license section must state MIT OR Apache-2.0
- The XCFramework binary is a compiled artifact — it carries Onde's license, mistral.rs attribution, and the model license notice
- The GitHub Release notes for every onde-swift release must include a `## Licenses` section

### Apple App Store

- Apple's review guidelines do not restrict MIT/Apache 2.0 licensed code
- The Qwen model license does not conflict with App Store distribution
- Ensure the app's **Privacy Policy** states that inference runs entirely on-device and no user data is transmitted (this is a functional claim Onde enables, not a license issue, but legal teams will ask)

---

## Checklist — Adding a New Model

When adding a new model to `src/inference/models.rs`:

- [ ] Verify the model's license on HuggingFace before adding
- [ ] Check for the >100M MAU commercial threshold clause
- [ ] Add the model to the `## Model Licenses` table in `onde/README.md`
- [ ] Add the model to the `## Model Licenses` table in `onde-swift/README.md`
- [ ] Add the model to the model license table in this SKILL.md
- [ ] If the model license is more restrictive than Qwen Community License, escalate before shipping

## Checklist — Adding a New Dependency

- [ ] Run `cargo license --all-features` and confirm no GPL appears
- [ ] If MPL-2.0: confirm you are not modifying the MPL files
- [ ] If Apache-2.0 only (not MIT): compatible, no action needed
- [ ] If custom/proprietary license: stop and escalate to Splitfire AB legal

## Checklist — New SDK Release

- [ ] Both `LICENSE-MIT` and `LICENSE-APACHE` are present
- [ ] `Cargo.toml` / `pubspec.yaml` / `package.json` declare `MIT OR Apache-2.0`
- [ ] README contains `## Model Licenses` section
- [ ] Copyright year and entity (`Onde Inference (Splitfire AB)`) is correct
- [ ] mistral.rs fork dependency is pinned to a commit SHA
- [ ] mistral.rs `LICENSE` (Eric Buehler) is intact and unmodified

---

## Quick Reference

| Question | Answer |
|---|---|
| What license is Onde? | MIT OR Apache-2.0 (user's choice) |
| Who holds the copyright? | Onde Inference (Splitfire AB) |
| Can someone fork Onde and sell it? | Yes, under MIT or Apache 2.0, but they cannot use the "Onde Inference" name or logo |
| Are Qwen models MIT? | No — Qwen Community License. Disclose to users. |
| Can enterprises use Onde commercially? | Yes. Apache 2.0 + Qwen Community License both permit commercial use. |
| Does the patent grant apply? | Yes, under Apache 2.0. Choose Apache 2.0 if patent protection matters to you. |
| Who is Eric Buehler? | Author of mistral.rs. His MIT copyright must be preserved in our fork. |