use crate::core::MnemeCore;
use crate::domain::*;
use tauri::State;

#[tauri::command]
pub fn get_config(core: State<'_, MnemeCore>) -> Result<AppConfigView, String> {
    core.config_view()
}
#[tauri::command]
pub fn set_config(core: State<'_, MnemeCore>, input: ConfigPatch) -> Result<AppConfigView, String> {
    core.set_config(input)
}
#[tauri::command]
pub fn list_sources(core: State<'_, MnemeCore>) -> Result<Vec<SourceMeta>, String> {
    core.vault.list_sources()
}
#[tauri::command]
pub fn get_source(core: State<'_, MnemeCore>, id: String) -> Result<SourceDetail, String> {
    let meta = core.vault.read_meta(&id)?.ok_or("source not found")?;
    Ok(SourceDetail {
        meta,
        content: core.vault.read_raw(&id)?,
        note: core.vault.read_note(&id)?,
    })
}
#[tauri::command]
pub async fn add_source(
    core: State<'_, MnemeCore>,
    input: AddSourceInput,
) -> Result<AddSourceResult, String> {
    core.add_source(input).await
}
#[tauri::command]
pub async fn wikify_source(core: State<'_, MnemeCore>, id: String) -> Result<WikifyReport, String> {
    core.wikify_source(&id).await
}
#[tauri::command]
pub fn list_entities(core: State<'_, MnemeCore>) -> Result<Vec<EntitySummary>, String> {
    core.vault.list_entities()
}
#[tauri::command]
pub fn get_entity(core: State<'_, MnemeCore>, slug: String) -> Result<EntityDetail, String> {
    let page = core.vault.read_entity(&slug)?.ok_or("entity not found")?;
    Ok(EntityDetail {
        backlinks: core.vault.backlinks(&slug)?,
        page,
    })
}
#[tauri::command]
pub fn search_entities(
    core: State<'_, MnemeCore>,
    query: String,
) -> Result<Vec<SearchHit>, String> {
    core.search(&query, 10)
}
#[tauri::command]
pub async fn research_query(
    core: State<'_, MnemeCore>,
    question: String,
) -> Result<ResearchResult, String> {
    core.research(&question).await
}
#[tauri::command]
pub fn get_graph(core: State<'_, MnemeCore>) -> Result<GraphData, String> {
    core.vault.graph()
}
#[tauri::command]
pub fn lint_vault(core: State<'_, MnemeCore>) -> Result<LintReport, String> {
    core.lint()
}
#[tauri::command]
pub fn sync_vault(core: State<'_, MnemeCore>) -> Result<SyncResult, String> {
    core.sync()
}
