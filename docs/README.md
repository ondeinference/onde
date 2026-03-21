# Onde Docs

Source for the [Onde documentation site](https://ondeinference.com/docs), built with [Mintlify](https://mintlify.com).

## Structure

```
docs/
├── mint.json          # Mintlify site config — navigation, colors, logo
├── index.md           # Landing page  →  /docs
├── dev.md             # Developer guide  →  /docs/dev
├── swift-package.md   # Swift Package guide  →  /docs/swift-package
├── ruby-gem.md        # Ruby Gem guide  →  /docs/ruby-gem
└── distribution.md    # Publishing guide  →  /docs/distribution
```

## Local Preview

Install the Mintlify CLI:

```bash
npm install -g mintlify
```

Start the dev server from the `docs/` directory:

```bash
cd docs
mintlify dev
```

The site is served at `http://localhost:3000`.

## Deployment

The docs deploy automatically via the [Mintlify GitHub App](https://github.com/apps/mintlify).

1. Install the Mintlify GitHub App on the `ondeinference/onde` repository.
2. In the Mintlify dashboard, set the docs directory to `docs/` and the base path to `/docs`.
3. Every push to `main` triggers a redeploy.

The custom domain is configured in the Mintlify dashboard:

- Domain: `ondeinference.com`
- Path: `/docs`

## Adding a Page

1. Create a new `.md` file in `docs/`.
2. Add a YAML frontmatter block at the top:
   ```yaml
   ---
   title: "Page Title"
   description: "One-sentence description shown in search and link previews."
   ---
   ```
3. Add the filename (without `.md`) to the appropriate group in `mint.json` under `navigation`.

## Frontmatter Fields

| Field | Required | Description |
| ----- | -------- | ----------- |
| `title` | Yes | Page title shown in the browser tab and sidebar |
| `description` | Yes | Meta description for search and link previews |
| `sidebarTitle` | No | Override the sidebar label (defaults to `title`) |

## Content Guidelines

- Write for developers who are evaluating or integrating Onde — assume Rust familiarity, not deep familiarity with the project.
- Code blocks must specify a language for syntax highlighting (` ```swift `, ` ```rust `, ` ```bash `, etc.).
- Cross-links between pages omit the `.md` extension: `[Swift Package](swift-package)`, not `[Swift Package](swift-package.md)`.
- Keep internal/contributor notes (build internals, pitfalls, architecture deep-dives) in the `.agent-skills/` directory, not in `docs/`.