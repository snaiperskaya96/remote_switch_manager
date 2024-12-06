use std::path::PathBuf;

pub fn get_storage_path() -> PathBuf {
    match std::env::args().nth(1)
    {
        Some(config_path) => {
            config_path.into()
        },
        None => {
            std::env::current_exe()
            .expect("Could not retrieve current_exe path")
            .parent()
            .expect("Could not retrieve parent's folder")
            .to_path_buf()
        },
    }
}