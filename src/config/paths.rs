use std::path::PathBuf;

pub fn config_dir() -> PathBuf {
    std::env::var_os("KDC_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".kdc")))
        .unwrap_or_else(|| PathBuf::from(".kdc"))
}

pub fn config_file() -> PathBuf {
    config_dir().join("config.yaml")
}

pub fn history_file() -> PathBuf {
    config_dir().join("history.yaml")
}
