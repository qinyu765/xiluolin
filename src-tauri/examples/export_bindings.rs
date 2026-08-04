use std::{env, path::PathBuf};

use specta_typescript::Typescript;

fn main() {
    let output = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("src/generated/tauri-bindings.ts"));

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).expect("无法创建绑定输出目录");
    }

    xiluolin_lib::bindings::builder()
        .export(Typescript::default(), &output)
        .expect("无法生成 Tauri TypeScript 绑定");
}
