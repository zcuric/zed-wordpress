# WordPress Coding Standards

Live [WordPress Coding Standards](https://github.com/WordPress/WordPress-Coding-Standards)
(phpcs / WPCS) diagnostics for PHP in [Zed](https://zed.dev).

A thin wrapper around [`efm-langserver`](https://github.com/mattn/efm-langserver)
that surfaces `phpcs` warnings as live LSP diagnostics. Pairs well with the
**WordPress Intelephense** extension, which provides completion, hover, and
go-to-definition.

## Activate

Add `wpcs` to the PHP `language_servers` list in your Zed `settings.json`
(or a per-project `.zed/settings.json`):

```json
{
  "languages": {
    "PHP": {
      "language_servers": ["wp-intelephense", "wpcs", "..."]
    }
  }
}
```

## Requirements

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
a clear message pointing at the install command.

## Formatting with phpcbf (optional, external)

This extension reports diagnostics; for auto-fixing on save, configure
`phpcbf` as an external formatter:

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

## License

Apache-2.0. Source: <https://github.com/zcuric/zed-wordpress>.
