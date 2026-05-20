use std::{env, fs};

use zed_extension_api::settings::LspSettings;
use zed_extension_api::{
    self as zed, serde_json, CodeLabel, CodeLabelSpan, LanguageServerId, Result,
};

const LANGUAGE_SERVER_ID: &str = "wp-intelephense";

const NPM_PACKAGE: &str = "intelephense";
const NPM_SERVER_PATH: &str = "node_modules/intelephense/lib/intelephense.js";

const DEFAULT_STUBS: &[&str] = &[
    // PHP core extensions (mirrors bitpoke/wordpress.nvim).
    "apache",
    "bcmath",
    "bz2",
    "calendar",
    "com_dotnet",
    "Core",
    "ctype",
    "curl",
    "date",
    "dba",
    "dom",
    "enchant",
    "exif",
    "FFI",
    "fileinfo",
    "filter",
    "fpm",
    "ftp",
    "gd",
    "gettext",
    "gmp",
    "hash",
    "iconv",
    "imap",
    "intl",
    "json",
    "ldap",
    "libxml",
    "mbstring",
    "meta",
    "mysqli",
    "oci8",
    "odbc",
    "openssl",
    "pcntl",
    "pcre",
    "PDO",
    "pdo_ibm",
    "pdo_mysql",
    "pdo_pgsql",
    "pdo_sqlite",
    "pgsql",
    "Phar",
    "posix",
    "pspell",
    "readline",
    "Reflection",
    "session",
    "shmop",
    "SimpleXML",
    "snmp",
    "soap",
    "sockets",
    "sodium",
    "SPL",
    "sqlite3",
    "standard",
    "superglobals",
    "sysvmsg",
    "sysvsem",
    "sysvshm",
    "tidy",
    "tokenizer",
    "xml",
    "xmlreader",
    "xmlrpc",
    "xmlwriter",
    "xsl",
    "Zend OPcache",
    "zip",
    "zlib",
    // WordPress ecosystem (real php-stubs/* packages bundled with Intelephense).
    "wordpress",
    "wordpress-globals",
    "wp-cli",
    "woocommerce",
    "acf-pro",
    "genesis",
    "polylang",
];

struct WordPressIntelephenseExtension {
    installed: bool,
}

impl WordPressIntelephenseExtension {
    fn command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        if let Some(path) = worktree.which("intelephense") {
            return Ok(zed::Command {
                command: path,
                args: vec!["--stdio".to_string()],
                env: Default::default(),
            });
        }

        let server_path = self.ensure_installed(language_server_id)?;
        Ok(zed::Command {
            command: zed::node_binary_path()?,
            args: vec![
                env::current_dir()
                    .map_err(|e| e.to_string())?
                    .join(&server_path)
                    .to_string_lossy()
                    .to_string(),
                "--stdio".to_string(),
            ],
            env: Default::default(),
        })
    }

    fn ensure_installed(&mut self, language_server_id: &LanguageServerId) -> Result<String> {
        let server_exists = fs::metadata(NPM_SERVER_PATH).is_ok_and(|stat| stat.is_file());

        if self.installed && server_exists {
            return Ok(NPM_SERVER_PATH.to_string());
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let version = zed::npm_package_latest_version(NPM_PACKAGE)?;
        if !server_exists
            || zed::npm_package_installed_version(NPM_PACKAGE)?.as_deref() != Some(&version)
        {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            match zed::npm_install_package(NPM_PACKAGE, &version) {
                Ok(()) => {
                    if !fs::metadata(NPM_SERVER_PATH).is_ok_and(|stat| stat.is_file()) {
                        return Err(format!(
                            "installed package '{NPM_PACKAGE}' did not contain expected path '{NPM_SERVER_PATH}'",
                        ));
                    }
                }
                Err(err) => {
                    if !fs::metadata(NPM_SERVER_PATH).is_ok_and(|stat| stat.is_file()) {
                        return Err(err);
                    }
                }
            }
        }

        self.installed = true;
        Ok(NPM_SERVER_PATH.to_string())
    }
}

fn default_settings() -> serde_json::Value {
    serde_json::json!({
        "stubs": DEFAULT_STUBS,
        "files": {
            "maxSize": 5_000_000
        }
    })
}

fn deep_merge(base: &mut serde_json::Value, overlay: serde_json::Value) {
    match (base, overlay) {
        (serde_json::Value::Object(base_map), serde_json::Value::Object(overlay_map)) => {
            for (key, value) in overlay_map {
                let slot = base_map.entry(key).or_insert(serde_json::Value::Null);
                deep_merge(slot, value);
            }
        }
        (slot, value) => {
            *slot = value;
        }
    }
}

/// Format Intelephense completion items more usefully — method signatures,
/// property types, dimmed system variables. Lifted from `zed-extensions/php`.
fn label_for_completion(completion: zed::lsp::Completion) -> Option<CodeLabel> {
    let label = &completion.label;
    let kind = completion.kind?;

    match kind {
        zed::lsp::CompletionKind::Method => {
            if let Some(detail) = completion.detail.as_deref() {
                if detail.is_empty() {
                    return Some(CodeLabel {
                        spans: vec![
                            CodeLabelSpan::literal(label, Some("function.method".into())),
                            CodeLabelSpan::literal("()", None),
                        ],
                        filter_range: (0..label.len()).into(),
                        code: completion.label,
                    });
                }
            }

            let detail = completion.detail.as_deref()?;
            let mut parts = detail.split(':');
            let name_and_params = parts.next()?;
            let return_type = parts.next()?.trim();
            let (_, params) = name_and_params.split_once('(')?;
            let params = params.trim_end_matches(')');

            Some(CodeLabel {
                spans: vec![
                    CodeLabelSpan::literal(label, Some("function.method".into())),
                    CodeLabelSpan::literal("(", None),
                    CodeLabelSpan::literal(params, Some("comment".into())),
                    CodeLabelSpan::literal("): ", None),
                    CodeLabelSpan::literal(return_type, Some("type".into())),
                ],
                filter_range: (0..label.len()).into(),
                code: completion.label,
            })
        }
        zed::lsp::CompletionKind::Constant | zed::lsp::CompletionKind::EnumMember => {
            if let Some(detail) = completion.detail.as_deref() {
                if !detail.is_empty() {
                    return Some(CodeLabel {
                        spans: vec![
                            CodeLabelSpan::literal(label, Some("constant".into())),
                            CodeLabelSpan::literal(" ", None),
                            CodeLabelSpan::literal(detail, Some("comment".into())),
                        ],
                        filter_range: (0..label.len()).into(),
                        code: completion.label,
                    });
                }
            }

            Some(CodeLabel {
                spans: vec![CodeLabelSpan::literal(label, Some("constant".into()))],
                filter_range: (0..label.len()).into(),
                code: completion.label,
            })
        }
        zed::lsp::CompletionKind::Property => {
            let return_type = completion.detail?;
            Some(CodeLabel {
                spans: vec![
                    CodeLabelSpan::literal(label, Some("attribute".into())),
                    CodeLabelSpan::literal(": ", None),
                    CodeLabelSpan::literal(return_type, Some("type".into())),
                ],
                filter_range: (0..label.len()).into(),
                code: completion.label,
            })
        }
        zed::lsp::CompletionKind::Variable => {
            // https://www.php.net/manual/en/reserved.variables.php
            const SYSTEM_VAR_NAMES: &[&str] =
                &["argc", "argv", "php_errormsg", "http_response_header"];

            let var_name = completion.label.trim_start_matches('$');
            let is_uppercase = var_name
                .chars()
                .filter(|c| c.is_alphabetic())
                .all(|c| c.is_uppercase());
            let is_system_constant = var_name.starts_with('_');
            let is_reserved = SYSTEM_VAR_NAMES.contains(&var_name);

            let highlight =
                (is_uppercase || is_system_constant || is_reserved).then(|| "comment".to_string());

            Some(CodeLabel {
                spans: vec![CodeLabelSpan::literal(label, highlight)],
                filter_range: (0..label.len()).into(),
                code: completion.label,
            })
        }
        _ => None,
    }
}

impl zed::Extension for WordPressIntelephenseExtension {
    fn new() -> Self {
        Self { installed: false }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        match language_server_id.as_ref() {
            LANGUAGE_SERVER_ID => self.command(language_server_id, worktree),
            id => Err(format!("unknown language server: {id}")),
        }
    }

    fn language_server_initialization_options(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        if language_server_id.as_ref() != LANGUAGE_SERVER_ID {
            return Ok(None);
        }

        let init_options = LspSettings::for_worktree(LANGUAGE_SERVER_ID, worktree)
            .ok()
            .and_then(|settings| settings.initialization_options);

        Ok(init_options)
    }

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        if language_server_id.as_ref() != LANGUAGE_SERVER_ID {
            return Ok(None);
        }

        let mut intelephense = default_settings();
        if let Ok(settings) = LspSettings::for_worktree(LANGUAGE_SERVER_ID, worktree) {
            if let Some(user) = settings.settings {
                deep_merge(&mut intelephense, user);
            }
        }

        Ok(Some(serde_json::json!({ "intelephense": intelephense })))
    }

    fn label_for_completion(
        &self,
        language_server_id: &LanguageServerId,
        completion: zed::lsp::Completion,
    ) -> Option<CodeLabel> {
        if language_server_id.as_ref() == LANGUAGE_SERVER_ID {
            label_for_completion(completion)
        } else {
            None
        }
    }
}

zed::register_extension!(WordPressIntelephenseExtension);
