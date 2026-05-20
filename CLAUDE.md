# zed-wordpress

## What this is

A monorepo of **two** [Zed](https://zed.dev) editor extensions for WordPress
and WooCommerce PHP work, published to the `zed-industries/extensions`
registry separately from this one repo (`zcuric/zed-wordpress`):

- **`wordpress-intelephense`** (`intelephense/` subdir) — a pre-configured
  wrapper around the Intelephense PHP language server. Same binary the
  built-in PHP extension uses, but shipped with
  WordPress/WooCommerce/ACF/Genesis/Polylang stubs, a larger `files.maxSize`,
  and prettier completion labels.
- **`wordpress-coding-standards`** (`wpcs/` subdir) — an `efm-langserver`
  wrapper that surfaces phpcs / WordPress Coding Standards diagnostics.

They were one combined extension until a Zed maintainer asked for the split
(PR `zed-industries/extensions#6144`) so users can install just one language
server at a time. Inspired by
[`bitpoke/wordpress.nvim`](https://github.com/bitpoke/wordpress.nvim).

## Non-goals

- **No bundled binaries.** Zed's extension policy forbids it. Intelephense
  installs from `npm` lazily on first run; `efm-langserver` and `phpcs` are
  the user's responsibility.
- **No tree-sitter grammar work.** The PHP grammar and language config are
  provided by the built-in PHP extension (`zed-extensions/php`). We only
  register additional language servers against the existing `PHP` language.
- **No single combined extension.** The maintainer explicitly wants two.
  Don't merge them back.

## Repository layout

One git repo, two independent extensions, each its own Rust crate:

```
zed-wordpress/
├── CLAUDE.md, README.md, LICENSE   shared, repo root
├── rust-toolchain.toml             shared (rustup searches upward)
├── intelephense/                   extension: wordpress-intelephense
│   ├── extension.toml
│   ├── Cargo.toml                  crate: wordpress_intelephense
│   └── src/intelephense.rs
└── wpcs/                           extension: wordpress-coding-standards
    ├── extension.toml
    ├── Cargo.toml                  crate: wordpress_coding_standards
    └── src/wpcs.rs
```

There is **no workspace `Cargo.toml`** — each subdir is a standalone crate so
the Zed registry can build each extension independently. The registry
references both via one shared submodule plus the `path` field in
`extensions.toml` (the main `zed` repo does the same for `glsl`/`html`/
`proto`):

```toml
[wordpress-coding-standards]
submodule = "extensions/wordpress"
path = "wpcs"
version = "0.1.0"

[wordpress-intelephense]
submodule = "extensions/wordpress"
path = "intelephense"
version = "0.1.0"
```

## Architecture

Each crate is a `cdylib` compiled to `wasm32-wasip1`, registering exactly
one language server.

### `wordpress-intelephense` — LS id `wp-intelephense`

`intelephense/src/intelephense.rs`. Discovers `intelephense` via
`worktree.which`; falls back to installing the `intelephense` npm package
into the extension's working directory and launching it via
`zed::node_binary_path()` (the upstream PHP extension pattern). Trait
methods:

- `language_server_command` — discovery + npm install fallback.
- `language_server_initialization_options` — pass-through of
  `lsp.wp-intelephense.initialization_options` (e.g. paid `licenceKey`).
- `language_server_workspace_configuration` — WordPress-flavored defaults
  (stubs + `files.maxSize`) deep-merged with `lsp.wp-intelephense.settings`,
  wrapped as `{ "intelephense": ... }`.
- `label_for_completion` — prettier method/property/variable/constant
  labels, lifted from upstream `zed-extensions/php`.

### `wordpress-coding-standards` — LS id `wpcs`

`wpcs/src/wpcs.rs`. Only `language_server_command`: wraps `efm-langserver`.
Looks for a project-local config at `<root>/.zed/efm-wp.yaml`; if present,
passes `-c <path>`. Otherwise launches `efm-langserver` with no args and
lets it find its default `~/.config/efm-langserver/config.yaml`. If
`efm-langserver` isn't on `PATH`, returns an actionable error pointing at
the install command.

The LS ids (`wp-intelephense`, `wpcs`) are deliberately unchanged from the
pre-split combined extension, so users' existing `.zed/settings.json` files
keep working.

## Default workspace configuration (`wordpress-intelephense`)

Minimal on purpose. Two things and nothing else:

- **`intelephense.stubs`** — PHP core extensions + `wordpress`,
  `wordpress-globals`, `wp-cli`, `woocommerce`, `acf-pro`, `genesis`,
  `polylang`. (Core list mirrors `wordpress.nvim`; the rest match real
  [php-stubs/](https://github.com/php-stubs) packages so Intelephense can
  resolve them.)
- **`intelephense.files.maxSize = 5_000_000`** — bumps the 1MB default so
  generated files (compiled translations, big plugin bundles) get indexed.

Anything else (`environment.phpVersion`, `diagnostics.*`, `completion.*`,
`format.*`) is left to Intelephense's defaults or to the user's own
`settings.json`. Keep this list tight — every default we add is a default we
have to maintain.

## Activation model

Users opt in via Zed `settings.json` (global) or a project-level
`.zed/settings.json` (recommended — keeps non-WP PHP projects on defaults):

```json
{
  "languages": {
    "PHP": {
      "language_servers": [
        "wp-intelephense",
        "wpcs",
        "!intelephense",
        "!phpactor",
        "!phptools",
        "..."
      ]
    }
  }
}
```

`wpcs` is only meaningful if the `wordpress-coding-standards` extension is
installed; drop it otherwise. All three other PHP servers from the built-in
PHP extension should be disabled. If `phpactor` stays on, it'll independently
report WordPress core functions like `locate_template` as
`worse.unresolved_name` because phpactor has no concept of Intelephense's
stubs — it only reads what's on disk. The `"..."` keeps any
non-PHP-extension language servers the user has added.

## Why we DON'T ship `languages/php/config.toml`

The built-in PHP extension (`zed-extensions/php`) already declares the `PHP`
language with its grammar, brackets, comment style, prettier hookup, etc.
Each `extension.toml` references that existing language via `language = "PHP"`
in its `[language_servers.*]` block. Redeclaring the language would either
conflict or shadow the built-in config — and we'd have to keep our copy in
sync with upstream forever. Don't add it back without a concrete reason.

## Coding conventions

- Rust 2021.
- `cargo fmt` and `cargo clippy --target wasm32-wasip1 -- -D warnings` must
  be clean **in each crate**.
- Build JSON config with `serde_json::json!` — never raw strings.
- Errors are `Result<T, String>` (the extension API's alias). Make error
  messages actionable — name the missing binary, suggest the install command.
- Pin `zed_extension_api` to an exact version, not a range. Upstream PHP
  extension uses `"0.7.0"` (no caret); match that — the API has had breaking
  changes and Zed's `is_supported_wasm_api_version` check is strict.
- Extension `id` / `name` must not contain `zed` or `extension` (registry
  rule). The repo name `zed-wordpress` is fine — that rule is about the
  `extension.toml` `id`/`name`, not the GitHub repo or Rust crate names.

## Dev loop

One-time setup:

```sh
rustup target add wasm32-wasip1
```

Iteration (per extension — they install separately):

1. Edit `intelephense/src/intelephense.rs` or `wpcs/src/wpcs.rs`.
2. In Zed: Command Palette → `zed: install dev extension` → pick the
   `intelephense/` or `wpcs/` subdirectory (not the repo root). After that,
   the "Rebuild" button in `zed: extensions` reloads in ~10–30s.
3. Logs: `zed: open log`, or launch `zed --foreground` from a terminal.

To build outside Zed for sanity checks, run inside the relevant subdir:

```sh
cargo build --target wasm32-wasip1 --release
```

## Reference repos (priority order)

1. **`zed-extensions/php`** — closest pattern. Especially
   `src/language_servers/intelephense.rs` (server discovery + workspace
   config merge) and `src/php.rs` (entry point / `Extension` trait impl).
2. **`bitpoke/wordpress.nvim`** — `lua/wordpress.lua` has the canonical stubs
   list to mirror.
3. **`zed-industries/extensions`** — registry conventions: `extensions.toml`
   entry format, the `path` field for multi-extension repos, the sorted-
   `extensions.toml`/`.gitmodules` CI check, the CLA, and the dangerfile.

## Verification checklist (before any commit)

Run in **each** crate directory (`intelephense/`, `wpcs/`):

- [ ] `cargo fmt -- --check` clean.
- [ ] `cargo clippy --target wasm32-wasip1 -- -D warnings` clean.
- [ ] `cargo build --target wasm32-wasip1 --release` succeeds.
- [ ] Dev extension loads in Zed without errors in `zed: open log`.
- [ ] Opening a `.php` file in a WordPress project shows the expected
      server (`wp-intelephense` and/or `wpcs`) in Zed's LSP status bar.
- [ ] `wp-intelephense`: hover on `add_action`, `wc_get_product`,
      `get_field` resolves to stub definitions.
- [ ] `wpcs`: a WPCS violation surfaces as a diagnostic.

## v0.x ideas (not now)

- **ACF field completion via a slash command.** Read `acf-json/*.json` in
  the worktree and provide field-name completions through Zed's
  slash-command API. Would be a real differentiator over plain stubs since
  field names are project-specific.
- **Project-local stub generation for custom plugins.** Detect a
  `wp-content/plugins/<slug>` layout and synthesize a stub file (or point
  Intelephense at the plugin's PHP source).
- **Better WPCS UX.** Codegen the `efm-langserver` config from worktree
  state (Composer-resolved `installed_paths`, detected standard, etc.)
  rather than asking the user to author YAML by hand. Blocked on the
  extension API not exposing filesystem writes — would need a host-side
  helper.
