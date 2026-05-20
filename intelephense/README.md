# WordPress Intelephense

The [Intelephense](https://intelephense.com) PHP language server for
[Zed](https://zed.dev), pre-configured for WordPress and WooCommerce
development.

Same Intelephense binary the built-in PHP extension uses, but with:

- WordPress, WordPress-Globals, WP-CLI, WooCommerce, ACF Pro, Genesis, and
  Polylang stubs enabled by default.
- `files.maxSize` raised to 5 MB so larger plugin and generated files get
  indexed.
- Completion labels formatted with parameter and return-type hints.
- Automatic install of the Intelephense `npm` package when it isn't already
  on your `PATH`.

Pairs well with the **WordPress Coding Standards** extension, which adds
phpcs / WPCS diagnostics. Inspired by
[`bitpoke/wordpress.nvim`](https://github.com/bitpoke/wordpress.nvim).

## Activate

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

`!intelephense`, `!phpactor`, `!phptools` disable the other PHP servers from
the built-in PHP extension. Leaving them on means duplicate or conflicting
diagnostics — e.g. `phpactor` flagging `locate_template` as
`worse.unresolved_name` because it doesn't read Intelephense's stubs.

## Requirements

Intelephense isn't bundled (Zed extensions can't ship binaries). If it's on
your `PATH` it's used as-is; otherwise the extension installs the `npm`
package on first launch.

## Configuration

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

## License

Apache-2.0. Source: <https://github.com/zcuric/zed-wordpress>.
