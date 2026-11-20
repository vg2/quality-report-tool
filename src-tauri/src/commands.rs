//! IPC commands. All inputs from the webview are untrusted: they are parsed
//! into validated domain types before reaching storage.

use crate::error::CommandError;
use crate::AppState;
use qrt_domain::{Name, System, Workspace, WorkspaceId};
use qrt_store::{SystemRepository, WorkspaceRegistry};
use tauri::State;

pub const MAX_DESCRIPTION_CHARS: usize = 2000;

fn parse_name(raw: &str) -> Result<Name, CommandError> {
    Name::parse(raw).map_err(|e| CommandError::invalid_input(capitalise(&e.to_string())))
}

fn parse_workspace_id(raw: &str) -> Result<WorkspaceId, CommandError> {
    raw.parse()
        .map_err(|_| CommandError::invalid_input("Invalid workspace id."))
}

/// Optional free text: trimmed, length-limited, no control characters other
/// than newlines and tabs. Empty becomes `None`.
pub fn parse_description(raw: Option<String>) -> Result<Option<String>, CommandError> {
    let Some(raw) = raw else { return Ok(None) };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.chars().count() > MAX_DESCRIPTION_CHARS {
        return Err(CommandError::invalid_input(format!(
            "Description must be at most {MAX_DESCRIPTION_CHARS} characters."
        )));
    }
    if trimmed
        .chars()
        .any(|c| c.is_control() && c != '\n' && c != '\t')
    {
        return Err(CommandError::invalid_input(
            "Description contains invalid characters.",
        ));
    }
    Ok(Some(trimmed.to_owned()))
}

fn capitalise(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => format!("{}{}.", first.to_uppercase(), chars.as_str()),
        None => String::new(),
    }
}

#[tauri::command(async)]
pub fn list_workspaces(state: State<'_, AppState>) -> Result<Vec<Workspace>, CommandError> {
    tracing::debug!("ipc: list_workspaces");
    Ok(state.store.list_workspaces()?)
}

#[tauri::command(async)]
pub fn create_workspace(
    state: State<'_, AppState>,
    name: String,
) -> Result<Workspace, CommandError> {
    Ok(state.store.create_workspace(parse_name(&name)?)?)
}

#[tauri::command(async)]
pub fn list_systems(
    state: State<'_, AppState>,
    workspace_id: String,
) -> Result<Vec<System>, CommandError> {
    let id = parse_workspace_id(&workspace_id)?;
    Ok(state.store.open_workspace(id)?.list_systems()?)
}

#[tauri::command(async)]
pub fn create_system(
    state: State<'_, AppState>,
    workspace_id: String,
    name: String,
    description: Option<String>,
) -> Result<System, CommandError> {
    let id = parse_workspace_id(&workspace_id)?;
    let name = parse_name(&name)?;
    let description = parse_description(description)?;
    Ok(state
        .store
        .open_workspace(id)?
        .create_system(name, description)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_errors_are_user_facing() {
        assert_eq!(
            parse_name("  ").err(),
            Some(CommandError::invalid_input("Name must not be empty."))
        );
    }

    #[test]
    fn workspace_id_must_be_uuid() {
        assert!(parse_workspace_id("../../etc/passwd").is_err());
        assert!(parse_workspace_id(&WorkspaceId::new().to_string()).is_ok());
    }

    #[test]
    fn description_rules() {
        assert_eq!(parse_description(None), Ok(None));
        assert_eq!(parse_description(Some("   ".into())), Ok(None));
        assert_eq!(
            parse_description(Some(" a\nb ".into())),
            Ok(Some("a\nb".into()))
        );
        assert!(parse_description(Some("bad\u{7}".into())).is_err());
        assert!(parse_description(Some("x".repeat(MAX_DESCRIPTION_CHARS + 1))).is_err());
    }
}
