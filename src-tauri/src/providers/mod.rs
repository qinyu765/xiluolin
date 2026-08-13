pub mod asr;
mod asr_adapters;
pub mod catalog;
pub mod error;
pub mod text;
mod text_adapters;
mod transport;

pub use catalog::provider_catalog;

#[tauri::command]
#[specta::specta]
pub fn list_provider_catalog() -> catalog::ProviderCatalog {
    provider_catalog()
}
