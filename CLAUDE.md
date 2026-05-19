# zed-wordpress

## What this is

A [Zed](https://zed.dev) editor extension that adds `wp-intelephense` — a
pre-configured wrapper around the Intelephense PHP language server tuned for
WordPress and WooCommerce work. Same binary the built-in PHP extension uses,
but shipped with WordPress/WooCommerce/ACF/Genesis/Polylang stubs and a larger
`files.maxSize` so Intelephense actually indexes plugin-sized files.

Distributed via the `zed-industries/extensions` registry; repo lives at
`castel-code/zed-wordpress`. Inspired by
[`bitpoke/wordpress.nvim`](https://github.com/bitpoke/wordpress.nvim).

## Non-goals (still, even at v0.2)

- **No bundled binaries.** Zed's extension policy forbids it. Intelephense
  installs from `npm` lazily on first run; `efm-langserver` and `phpcs` are
  the user's responsibility.
- **No tree-sitter grammar work.** The PHP grammar and language config are
  provided by the built-in PHP extension (`zed-extensions/php`). We only
  register additional language servers against the existing `PHP` language.

## Architecture (v0.2)

- Single Rust crate, `cdylib`, compiled to `wasm32-wasip1`.
- Two language server IDs:
  - **`wp-intelephense`** — discovers `intelephense` via
    `worktree.which`; falls back to installing the `intelephense` npm
    package into the extension's working directory and launching it via
    `zed::node_binary_path()` (the upstream PHP extension pattern). Three
    trait methods drive it:
    - `language_server_command` — discovery + install fallback.
    - `language_server_initialization_options` — pass-through of
      `lsp.wp-intelephense.initialization_options` (e.g. paid `licenceKey`).
    - `language_server_workspace_configuration` — WordPress-flavored defaults
      (stubs + `files.maxSize`) deep-merged with
      `lsp.wp-intelephense.settings`, wrapped as `{ "intelephense": ... }`.
    - `label_for_completion` — prettier method/property/variable/constant
      labels, lifted from upstream `zed-extensions/php`.
  - **`wpcs`** — wraps `efm-langserver` for phpcs/WPCS diagnostics. Looks
    for a project-local config at `<root>/.zed/efm-wp.yaml`; if present,
    passes `-c <path>`. Otherwise launches `efm-langserver` with no args and
    lets it find its default `~/.config/efm-langserver/config.yaml`. If
    `efm-langserver` isn't on `PATH`, returns an actionable error pointing
    at the install command.

The deep-merge helper lives in `src/wordpress.rs` — single file is still
fine at this size; split if a third language server or significant logic
ever shows up.

## Default workspace configuration

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
        "!intelephense",
        "!phpactor",
        "!phptools",
        "..."
      ]
    }
  }
}
```

All three other PHP servers from the built-in PHP extension must be disabled,
not just intelephense. If `phpactor` stays on, it'll independently report
WordPress core functions like `locate_template` as `worse.unresolved_name`
because phpactor has no concept of Intelephense's stubs — it only reads what's
on disk. The `"..."` keeps any non-PHP-extension language servers the user
has added. Document this prominently in the README.

## Why we DON'T ship `languages/php/config.toml`

The built-in PHP extension (`zed-extensions/php`) already declares the `PHP`
language with its grammar, brackets, comment style, prettier hookup, etc.
Our `extension.toml` references that existing language via `language = "PHP"`
in `[language_servers.wp-intelephense]`. Redeclaring the language here would
either conflict or shadow the built-in config — and we'd have to keep our
copy in sync with upstream forever. Don't add it back without a concrete
reason.

## Coding conventions

- Rust 2021.
- `cargo fmt` and `cargo clippy --all-targets -- -D warnings` must be clean.
- Build JSON config with `serde_json::json!` — never raw strings.
- Errors are `Result<T, String>` (the extension API's alias). Make error
  messages actionable — name the missing binary, suggest the install command.
- Pin `zed_extension_api` to an exact version, not a range. Upstream PHP
  extension uses `"0.7.0"` (no caret); match that — the API has had breaking
  changes and Zed's `is_supported_wasm_api_version` check is strict.

## Dev loop

One-time setup:

```sh
rustup target add wasm32-wasip1
```

Iteration:

1. Edit `src/wordpress.rs`.
2. In Zed: Command Palette → `zed: install dev extension` → pick this folder
   (only needed once; after that, the "Rebuild" button in
   `zed: extensions` reloads in ~10–30s).
3. Logs: `zed: open log`, or launch `zed --foreground` from a terminal for
   live output.

To build outside Zed for sanity checks:

```sh
cargo build --target wasm32-wasip1 --release
```

## Reference repos (priority order)

1. **`zed-extensions/php`** — closest pattern. Especially
   `src/language_servers/intelephense.rs` (server discovery + workspace
   config merge) and `src/php.rs` (entry point / `Extension` trait impl).
2. **`bitpoke/wordpress.nvim`** — `lua/wordpress.lua` has the canonical stubs
   list to mirror.
3. **`zed-industries/extensions`** — registry conventions for
   `extension.toml`, version bumps, and PR format when publishing.

## Verification checklist (before any commit)

- [ ] `cargo fmt -- --check` clean.
- [ ] `cargo clippy --all-targets -- -D warnings` clean.
- [ ] `cargo build --target wasm32-wasip1 --release` succeeds.
- [ ] Dev extension loads in Zed without errors in `zed: open log`.
- [ ] Opening a `.php` file in a WordPress project shows `wp-intelephense`
      in Zed's LSP status bar.
- [ ] Hover on `add_action`, `wc_get_product`, `get_field` resolves to stub
      definitions (proves the stubs config reached Intelephense).

## v0.3+ ideas (not now)

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
