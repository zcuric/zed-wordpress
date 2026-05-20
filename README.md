# zed-wordpress

WordPress / WooCommerce tooling for the [Zed](https://zed.dev) editor,
delivered as **two independent extensions** — install whichever you want:

- **WordPress Intelephense** (`wordpress-intelephense`) — the Intelephense
  PHP language server, pre-configured with WordPress, WordPress-Globals,
  WP-CLI, WooCommerce, ACF Pro, Genesis, and Polylang stubs, plus
  `files.maxSize` raised to 5 MB. Completion labels are formatted with
  parameter and return-type hints. Auto-installs Intelephense from `npm` if
  it isn't already on your `PATH`.
- **WordPress Coding Standards** (`wordpress-coding-standards`) — a thin
  wrapper around [`efm-langserver`](https://github.com/mattn/efm-langserver)
  that surfaces [WordPress Coding Standards](https://github.com/WordPress/WordPress-Coding-Standards)
  diagnostics from `phpcs` as live LSP warnings.

Both register language servers against the `PHP` language from Zed's
built-in PHP extension. Inspired by
[`bitpoke/wordpress.nvim`](https://github.com/bitpoke/wordpress.nvim).

This repository contains both extensions as subdirectories
(`intelephense/` and `wpcs/`); each is published to the Zed registry
separately.

---

## WordPress Intelephense

### Install

Command Palette → `zed: extensions` → search **WordPress Intelephense**.

### Activate

Add to your Zed `settings.json` (global) or a per-project
`.zed/settings.json` (recommended — keeps non-WordPress PHP projects on
their defaults):

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

`!intelephense`, `!phpactor`, `!phptools` disable the other PHP servers
from the built-in PHP extension. Leaving them on means duplicate or
conflicting diagnostics — e.g. `phpactor` flagging `locate_template` as
`worse.unresolved_name` because it doesn't read Intelephense's stubs.

### Requirements

Intelephense isn't bundled (extensions can't ship binaries). If it's on
your `PATH` it's used as-is; otherwise the extension installs the npm
package on first launch.

### Configuration

Override or extend the defaults under `lsp.wp-intelephense`. Anything under
`settings` deep-merges on top of the extension's defaults and is forwarded
to Intelephense:

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

- `settings.stubs`, if provided, **replaces** the default stubs list — copy
  the defaults if you want to extend rather than replace.
- `initialization_options.licenceKey` is passed straight through for paid
  Intelephense licences.

### Default stubs

PHP core extensions (`apache`, `bcmath`, `Core`, `curl`, `date`, `mbstring`,
`mysqli`, `PDO`, `pcre`, `Reflection`, `SPL`, `standard`, `superglobals`, …)
plus `wordpress`, `wordpress-globals`, `wp-cli`, `woocommerce`, `acf-pro`,
`genesis`, and `polylang` — all real
[`php-stubs/*`](https://github.com/php-stubs) packages bundled with
Intelephense.

---

## WordPress Coding Standards

### Install

Command Palette → `zed: extensions` → search **WordPress Coding
Standards**.

### Activate

Add `wpcs` to the PHP `language_servers` list:

```json
{
  "languages": {
    "PHP": {
      "language_servers": ["wp-intelephense", "wpcs", "..."]
    }
  }
}
```

### Requirements

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

3. **Give `efm-langserver` the WPCS config.** Either globally at
   `~/.config/efm-langserver/config.yaml`:

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
   extension detects it and passes `-c <path>` to `efm-langserver`. Handy
   for switching standards (`WordPress-Core` vs full `WordPress`) per
   project without touching your global config.

If `efm-langserver` isn't on `PATH`, the `wpcs` server fails to start with
a clear message.

### Formatting with phpcbf (optional, external)

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

---

## License

Apache-2.0.
