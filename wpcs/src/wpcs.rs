use std::fs;

use zed_extension_api::{self as zed, LanguageServerId, Result};

const LANGUAGE_SERVER_ID: &str = "wpcs";

/// Project-local `efm-langserver` config, relative to the worktree root.
const EFM_PROJECT_CONFIG: &str = ".zed/efm-wp.yaml";

struct WordPressCodingStandardsExtension;

impl WordPressCodingStandardsExtension {
    fn command(&self, worktree: &zed::Worktree) -> Result<zed::Command> {
        let efm = worktree.which("efm-langserver").ok_or_else(|| {
            "`efm-langserver` was not found on PATH. Install it (Go): \
             `go install github.com/mattn/efm-langserver@latest`, then add the \
             WPCS config from the WordPress Coding Standards extension README to \
             `~/.config/efm-langserver/config.yaml`."
                .to_string()
        })?;

        // Prefer a project-local config if one exists; otherwise let
        // efm-langserver discover its own default config.
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

impl zed::Extension for WordPressCodingStandardsExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        match language_server_id.as_ref() {
            LANGUAGE_SERVER_ID => self.command(worktree),
            id => Err(format!("unknown language server: {id}")),
        }
    }
}

zed::register_extension!(WordPressCodingStandardsExtension);
