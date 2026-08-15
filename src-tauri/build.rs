use std::{env, fs, path::PathBuf};

fn main() {
    stage_windows_runtime().expect("无法准备 Windows sherpa-onnx 运行库");
    tauri_build::build()
}

fn stage_windows_runtime() -> Result<(), Box<dyn std::error::Error>> {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return Ok(());
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").ok_or("缺少 OUT_DIR")?);
    let profile_dir = out_dir
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .ok_or("无法从 OUT_DIR 推导 Cargo profile 目录")?;
    let runtime_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("windows/runtime");

    fs::create_dir_all(&runtime_dir)?;
    if !profile_dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(profile_dir)? {
        let path = entry?.path();
        if path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("dll"))
        {
            let file_name = path
                .file_name()
                .ok_or("Windows 运行库路径缺少文件名")?
                .to_owned();
            fs::copy(&path, runtime_dir.join(file_name))?;
        }
    }

    Ok(())
}
