use crate::network::utils::req_json;

#[tauri::command]
#[specta::specta]
pub async fn fetch_active_years() -> Result<Vec<i32>, String> {
    req_json::<_, ()>("insights/years", None).await
}
