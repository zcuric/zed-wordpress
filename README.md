# zed-wordpress

WordPress / WooCommerce tooling for the [Zed](https://zed.dev) editor,
delivered as **two independent extensions** — install whichever you want:

| Extension | Directory | What it does |
|-----------|-----------|--------------|
| [**WordPress Intelephense**](intelephense/README.md) (`wordpress-intelephense`) | [`intelephense/`](intelephense/) | Intelephense PHP language server pre-configured with WordPress/WooCommerce/ACF/Genesis/Polylang stubs, a larger `files.maxSize`, and prettier completion labels. |
| [**WordPress Coding Standards**](wpcs/README.md) (`wordpress-coding-standards`) | [`wpcs/`](wpcs/) | Live WordPress Coding Standards (phpcs / WPCS) diagnostics via `efm-langserver`. |

Both register language servers against the `PHP` language from Zed's
built-in PHP extension. Inspired by
[`bitpoke/wordpress.nvim`](https://github.com/bitpoke/wordpress.nvim).

See each extension's own README for installation, activation, and
configuration:

- [`intelephense/README.md`](intelephense/README.md)
- [`wpcs/README.md`](wpcs/README.md)

## Repository layout

One repository, two independent extensions, each its own `cdylib` crate
compiled to `wasm32-wasip1`:

```
zed-wordpress/
├── intelephense/   extension: wordpress-intelephense   (LS: wp-intelephense)
└── wpcs/           extension: wordpress-coding-standards (LS: wpcs)
```

Each is published to the `zed-industries/extensions` registry separately.
Contributor notes live in [`CLAUDE.md`](CLAUDE.md).

## License

Apache-2.0.
