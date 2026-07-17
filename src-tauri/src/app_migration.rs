use std::{fs, path::Path};

use tauri::Manager;

const LEGACY_APP_IDENTIFIER: &str = "com.xiluolin.app";

pub fn migrate_legacy_app_data(app: &tauri::AppHandle) -> Result<(), String> {
    let destination = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    let Some(parent) = destination.parent() else {
        return Ok(());
    };
    let source = parent.join(LEGACY_APP_IDENTIFIER);
    copy_directory_if_needed(&source, &destination)
}

fn copy_directory_if_needed(source: &Path, destination: &Path) -> Result<(), String> {
    if !source.is_dir() || directory_has_entries(destination)? {
        return Ok(());
    }

    if let Err(error) = copy_directory(source, destination) {
        let _ = fs::remove_dir_all(destination);
        return Err(format!("迁移旧版应用数据失败：{error}"));
    }
    Ok(())
}

fn directory_has_entries(path: &Path) -> Result<bool, String> {
    if !path.exists() {
        return Ok(false);
    }
    path.read_dir()
        .map(|mut entries| entries.next().is_some())
        .map_err(|error| format!("检查应用数据目录失败：{error}"))
}

fn copy_directory(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_directory(&source_path, &destination_path)?;
        } else {
            fs::copy(source_path, destination_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn temp_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("xiluolin-{name}-{}", Uuid::new_v4()))
    }

    #[test]
    fn copies_legacy_data_without_removing_source() {
        let source = temp_path("legacy");
        let destination = temp_path("current");
        fs::create_dir_all(source.join("models")).unwrap();
        fs::write(source.join("settings.json"), "settings").unwrap();
        fs::write(source.join("models/model.bin"), "model").unwrap();

        copy_directory_if_needed(&source, &destination).unwrap();

        assert_eq!(
            fs::read_to_string(destination.join("settings.json")).unwrap(),
            "settings"
        );
        assert_eq!(
            fs::read_to_string(destination.join("models/model.bin")).unwrap(),
            "model"
        );
        assert!(source.join("settings.json").exists());
        let _ = fs::remove_dir_all(source);
        let _ = fs::remove_dir_all(destination);
    }

    #[test]
    fn existing_destination_is_not_overwritten() {
        let source = temp_path("legacy-existing");
        let destination = temp_path("current-existing");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&destination).unwrap();
        fs::write(source.join("settings.json"), "old").unwrap();
        fs::write(destination.join("settings.json"), "new").unwrap();

        copy_directory_if_needed(&source, &destination).unwrap();

        assert_eq!(
            fs::read_to_string(destination.join("settings.json")).unwrap(),
            "new"
        );
        let _ = fs::remove_dir_all(source);
        let _ = fs::remove_dir_all(destination);
    }
}
