# zed-wordpress

A [Zed](https://zed.dev) extension that turns the editor into a proper
WordPress / WooCommerce development environment.

It registers two PHP language servers, ready to use:

- **`wp-intelephense`** — Intelephense, pre-configured with WordPress,
  WordPress-Globals, WP-CLI, WooCommerce, ACF Pro, Genesis, and Polylang
  stubs, plus `files.maxSize` bumped to 5 MB so generated and plugin files
  actually get indexed. Auto-installs the Intelephense package via `npm` if
  it isn't already on your `PATH`. Completion labels are formatted with
  parameter and return-type hints (lifted from the upstream
  `zed-extensions/php` PHP extension).
- **`wpcs`** — a thin wrapper around
  [`efm-langserver`](https://github.com/mattn/efm-langserver) that surfaces
  [WordPress Coding Standards](https://github.com/WordPress/WordPress-Coding-Standards)
  diagnostics from `phpcs` as live LSP warnings.

Inspired by
[`bitpoke/wordpress.nvim`](https://github.com/bitpoke/wordpress.nvim).

## Install the extension

Once published to the Zed extension registry: Command Palette →
`zed: extensions` → search **WordPress**.

## Activate the servers

Add this to your Zed `settings.json` (global) or, recommended, to a
per-project `.zed/settings.json` so non-WordPress PHP projects keep their
defaults:

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

- `wp-intelephense` — completion, hover, go-to-definition.
- `wpcs` — phpcs/WPCS diagnostics. Drop it from the list if you don't want
  it.
- `!intelephense`, `!phpactor`, `!phptools` — disable the other PHP servers
  that ship with the built-in PHP extension. Leaving them on means you'll
  get duplicate or conflicting diagnostics (e.g. `phpactor` flagging
  `locate_template` as `worse.unresolved_name` because it doesn't know
  about Intelephense's stubs).

## Configuring `wp-intelephense`

Intelephense isn't bundled (extensions can't ship binaries). If you already
have it on `PATH` it's used as-is; if not, the extension installs the npm
package on first launch.

To extend or override the defaults, add `lsp.wp-intelephense` to your
settings. The contents under `settings` deep-merge on top of the extension's
defaults and are forwarded to Intelephense:

```json
{
  "lsp": {
    "wp-intelephense": {
      "settings": {
        "stubs": ["Core", "wordpress", "woocommerce", "my-plugin-stubs"],
        "files": { "maxSize": 8000000 },
        "environment": { "phpVersion": "8.2.0" }
      },
      "initialization_options": {
        "licenceKey": "YOUR-INTELEPHENSE-LICENCE-KEY"
      }
    }
  }
}
```

Notes:

- `settings.stubs`, if provided, **replaces** the default stubs list — copy
  the defaults if you want to extend rather than replace.
- `initialization_options.licenceKey` is the standard place for paid
  Intelephense licence keys; it's passed straight through to the server.

### Default stubs

PHP core extensions (`apache`, `bcmath`, `Core`, `curl`, `date`, `mbstring`,
`mysqli`, `PDO`, `pcre`, `Reflection`, `SPL`, `standard`, `superglobals`, …)
plus:

- `wordpress`, `wordpress-globals`, `wp-cli`
- `woocommerce`
- `acf-pro`
- `genesis`
- `polylang`

These match real
[`php-stubs/*`](https://github.com/php-stubs) packages, which Intelephense
ships with.

## Configuring `wpcs`

Three one-time steps on your machine:

1. **Install `efm-langserver`:**

   ```sh
   go install github.com/mattn/efm-langserver@latest
   ```

2. **Install `phpcs` + WPCS via Composer (global):**

   ```sh
   composer global require "squizlabs/php_codesniffer=*"
   composer global require --dev wp-coding-standards/wpcs
   phpcs --config-set installed_paths "$HOME/.composer/vendor/wp-coding-standards/wpcs"
   phpcs -i   # should list "WordPress"
   ```

3. **Drop the WPCS config into `efm-langserver`.** Either as a global default
   at `~/.config/efm-langserver/config.yaml`:

   ```yaml
   version: 2
   root-markers:
     - .git/
     - composer.json
   tools:
     php-phpcs-wp: &php-phpcs-wp
       lint-command: "phpcs -q --report=emacs --standard=WordPress -"
       lint-stdin: true
       lint-formats:
         - "%f:%l:%c: %trror - %m"
         - "%f:%l:%c: %tarning - %m"
   languages:
     php:
       - <<: *php-phpcs-wp
   ```

   Or as a project-local override at `<project>/.zed/efm-wp.yaml` — the
   extension picks it up automatically and passes `-c <path>` to
   `efm-langserver`. Useful for switching standards (e.g. `WordPress-Core`
   vs full `WordPress`) without touching your global config.

If `efm-langserver` isn't on `PATH`, the `wpcs` server will fail to start
with a clear message — leave `wpcs` out of `language_servers` to skip the
WPCS layer entirely.

## Formatting with phpcbf (optional, external)

`wpcs` reports diagnostics; for auto-fixing on save, configure `phpcbf` as
an external formatter:

```json
{
  "languages": {
    "PHP": {
      "formatter": {
        "external": {
          "command": "phpcbf",
          "arguments": ["--standard=WordPress", "-"]
        }
      },
      "format_on_save": "on"
    }
  }
}
```

`phpcbf` exits with code 1 on successful changes; Zed handles that
correctly for stdin-based formatters.

## Versioning

- **0.1.0** — pre-configured Intelephense for WordPress.
- **0.2.0** *(current)* — WPCS via `efm-langserver`, prettier completion
  labels, automatic Intelephense install via `npm` when missing from
  `PATH`.

## License

Apache-2.0.
