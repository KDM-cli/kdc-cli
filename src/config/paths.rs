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

pub fn deploy_history_file() -> PathBuf {
    config_dir().join("deploy_history.yaml")
}

pub fn doctor_report_file() -> PathBuf {
    config_dir().join("doctor_report.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deploy_history_path_is_under_config_dir() {
        let path = deploy_history_file();
        assert!(path.to_string_lossy().contains("deploy_history.yaml"));
    }

    #[test]
    fn doctor_report_path_is_under_config_dir() {
        let path = doctor_report_file();
        assert!(path.to_string_lossy().contains("doctor_report.json"));
    }
}
