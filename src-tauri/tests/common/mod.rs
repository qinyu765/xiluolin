use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use xiluolin_lib::data::LocalDatabase;

pub fn temp_db_path(test_name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("xiluolin-tests")
        .join(format!("{test_name}-{nanos}.sqlite"))
}

pub fn open_test_database(path: &Path) -> LocalDatabase {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("test database parent should be created");
    }

    let database = LocalDatabase::open(path).expect("test database should open");
    database
        .initialize()
        .expect("test database should initialize");
    database
}
