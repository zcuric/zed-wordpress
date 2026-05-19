use std::{env, fs};

use zed_extension_api::settings::LspSettings;
use zed_extension_api::{
    self as zed, serde_json, CodeLabel, CodeLabelSpan, LanguageServerId, Result,
};

const INTELEPHENSE_LS_ID: &str = "wp-intelephense";
const WPCS_LS_ID: &str = "wpcs";

const NPM_PACKAGE: &str = "intelephense";
const NPM_SERVER_PATH: &str = "node_modules/intelephense/lib/intelephense.js";

const EFM_PROJECT_CONFIG: &str = ".zed/efm-wp.yaml";

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

struct WordPressExtension {
    intelephense_installed: bool,
}

impl WordPressExtension {
    fn intelephense_command(
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

        let server_path = self.ensure_intelephense_installed(language_server_id)?;
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

    fn ensure_intelephense_installed(
        &mut self,
        language_server_id: &LanguageServerId,
    ) -> Result<String> {
        let server_exists = fs::metadata(NPM_SERVER_PATH).is_ok_and(|stat| stat.is_file());

        if self.intelephense_installed && server_exists {
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

        self.intelephense_installed = true;
        Ok(NPM_SERVER_PATH.to_string())
    }

    fn wpcs_command(&self, worktree: &zed::Worktree) -> Result<zed::Command> {
        let efm = worktree.which("efm-langserver").ok_or_else(|| {
            "`efm-langserver` was not found on PATH. Install it (Go): \
             `go install github.com/mattn/efm-langserver@latest`, then drop the \
             WPCS config from the zed-wordpress README into \
             `~/.config/efm-langserver/config.yaml`."
                .to_string()
        })?;

        let mut args = Vec::new();
        let project_config = format!("{}/{}", worktree.root_path(), EFM_PROJECT_CONFIG);
        if fs::metadata(&project_config).is_ok_and(|stat| stat.is_file()) {
            args.push("-c".to_string());
            args.push(project_config);
        }

        Ok(zed::Command {
            command: efm,
            args,
            env: Default::default(),
        })
    }
}

fn intelephense_default_settings() -> serde_json::Value {
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
fn label_for_intelephense_completion(completion: zed::lsp::Completion) -> Option<CodeLabel> {
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

impl zed::Extension for WordPressExtension {
    fn new() -> Self {
        Self {
            intelephense_installed: false,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        match language_server_id.as_ref() {
            INTELEPHENSE_LS_ID => self.intelephense_command(language_server_id, worktree),
            WPCS_LS_ID => self.wpcs_command(worktree),
            id => Err(format!("unknown language server: {id}")),
        }
    }

    fn language_server_initialization_options(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        // Only Intelephense uses init options (e.g. `licenceKey`).
        if language_server_id.as_ref() != INTELEPHENSE_LS_ID {
            return Ok(None);
        }

        let init_options = LspSettings::for_worktree(INTELEPHENSE_LS_ID, worktree)
            .ok()
            .and_then(|settings| settings.initialization_options);

        Ok(init_options)
    }

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        if language_server_id.as_ref() != INTELEPHENSE_LS_ID {
            return Ok(None);
        }

        let mut intelephense = intelephense_default_settings();
        if let Ok(settings) = LspSettings::for_worktree(INTELEPHENSE_LS_ID, worktree) {
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
        if language_server_id.as_ref() == INTELEPHENSE_LS_ID {
            label_for_intelephense_completion(completion)
        } else {
            None
        }
    }
}

zed::register_extension!(WordPressExtension);
