use std::{io, path::PathBuf, process::Command};

pub fn pick_folder() -> io::Result<Option<PathBuf>> {
    native_pick_folder()
}

#[cfg(target_os = "macos")]
fn native_pick_folder() -> io::Result<Option<PathBuf>> {
    let output = Command::new("osascript")
        .args([
            "-e",
            "POSIX path of (choose folder with prompt \"Select a project folder for KDC\")",
        ])
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok((!path.is_empty()).then(|| PathBuf::from(path)))
}

#[cfg(target_os = "windows")]
fn native_pick_folder() -> io::Result<Option<PathBuf>> {
    let script = r#"
Add-Type -AssemblyName System.Windows.Forms
$dialog = New-Object System.Windows.Forms.FolderBrowserDialog
$dialog.Description = 'Select a project folder for KDC'
if ($dialog.ShowDialog() -eq 'OK') { Write-Output $dialog.SelectedPath }
"#;
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok((!path.is_empty()).then(|| PathBuf::from(path)))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn native_pick_folder() -> io::Result<Option<PathBuf>> {
    let picker = if command_exists("zenity") {
        Some(("zenity", vec!["--file-selection", "--directory"]))
    } else if command_exists("kdialog") {
        Some(("kdialog", vec!["--getexistingdirectory"]))
    } else {
        None
    };

    let Some((command, args)) = picker else {
        return Ok(None);
    };

    let output = Command::new(command).args(args).output()?;
    if !output.status.success() {
        return Ok(None);
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok((!path.is_empty()).then(|| PathBuf::from(path)))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn command_exists(command: &str) -> bool {
    Command::new("sh")
        .args(["-c", &format!("command -v {command}")])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
