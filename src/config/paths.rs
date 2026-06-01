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

pub fn command_history_file() -> PathBuf {
    config_dir().join("command_history.yaml")
}

pub fn project_cache_file() -> PathBuf {
    config_dir().join("project_cache.yaml")
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

    #[test]
    fn command_history_path_is_under_config_dir() {
        let path = command_history_file();
        assert!(path.to_string_lossy().contains("command_history.yaml"));
    }
}
