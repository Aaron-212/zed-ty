use zed::LanguageServerId;
use zed_extension_api::{self as zed, Result, settings::LspSettings};

struct TyBinary {
    path: String,
    args: Option<Vec<String>>,
    environment: Option<Vec<(String, String)>>,
}

struct TyExtension {}

impl TyExtension {
    fn language_server_binary(
        &mut self,
        _: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<TyBinary> {
        let binary_settings = LspSettings::for_worktree("ty", worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.binary);
        let binary_args = binary_settings
            .as_ref()
            .and_then(|binary_settings| binary_settings.arguments.clone());

        let (platform, _) = zed::current_platform();
        let environment = match platform {
            zed::Os::Mac | zed::Os::Linux => Some(worktree.shell_env()),
            zed::Os::Windows => None,
        };

        if let Some(path) = binary_settings.and_then(|binary_settings| binary_settings.path) {
            return Ok(TyBinary {
                path,
                args: binary_args,
                environment,
            });
        }
        if let Some(path) = worktree.which("ty") {
            return Ok(TyBinary {
                path,
                args: binary_args,
                environment,
            });
        }

        Err("No binary found.
            Ty for Zed currently relies on external binary.
            Install one with `uv tool install ty`."
            .into())
    }
}

impl zed::Extension for TyExtension {
    fn new() -> Self {
        Self {}
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let ty_binary = self.language_server_binary(language_server_id, worktree)?;
        Ok(zed::Command {
            command: ty_binary.path,
            args: ty_binary.args.unwrap_or_else(|| vec!["server".into()]),
            env: ty_binary.environment.unwrap_or_default(),
        })
    }

    fn language_server_initialization_options(
        &mut self,
        server_id: &LanguageServerId,
        worktree: &zed_extension_api::Worktree,
    ) -> Result<Option<zed_extension_api::serde_json::Value>> {
        let settings = LspSettings::for_worktree(server_id.as_ref(), worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.initialization_options.clone())
            .unwrap_or_default();
        Ok(Some(settings))
    }

    fn language_server_workspace_configuration(
        &mut self,
        server_id: &LanguageServerId,
        worktree: &zed_extension_api::Worktree,
    ) -> Result<Option<zed_extension_api::serde_json::Value>> {
        let settings = LspSettings::for_worktree(server_id.as_ref(), worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings.clone())
            .unwrap_or_default();
        Ok(Some(settings))
    }
}

zed::register_extension!(TyExtension);
