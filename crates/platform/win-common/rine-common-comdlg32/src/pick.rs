use std::path::PathBuf;
#[cfg(target_pointer_width = "32")]
use std::process::Command;

pub fn pick_open(title: Option<String>, initial_dir: Option<String>) -> Option<PathBuf> {
    pick_path(title, initial_dir, false)
}

pub fn pick_save(title: Option<String>, initial_dir: Option<String>) -> Option<PathBuf> {
    pick_path(title, initial_dir, true)
}

fn pick_path(title: Option<String>, initial_dir: Option<String>, save: bool) -> Option<PathBuf> {
    if let Ok(path) = std::env::var("RINE_DIALOG_TEST_PATH")
        && !path.is_empty()
    {
        return Some(PathBuf::from(path));
    }

    pick_path_native(title, initial_dir, save)
}

#[cfg(target_pointer_width = "64")]
fn pick_path_native(
    title: Option<String>,
    initial_dir: Option<String>,
    save: bool,
) -> Option<PathBuf> {
    let mut dialog = rfd::FileDialog::new();
    if let Some(t) = title {
        dialog = dialog.set_title(&t);
    }
    if let Some(dir) = initial_dir {
        let p = PathBuf::from(dir);
        if p.is_dir() {
            dialog = dialog.set_directory(p);
        }
    }

    if save {
        dialog.save_file()
    } else {
        dialog.pick_file()
    }
}

#[cfg(target_pointer_width = "32")]
fn pick_path_native(
    title: Option<String>,
    initial_dir: Option<String>,
    save: bool,
) -> Option<PathBuf> {
    let mut backend_available = false;

    match pick_with_zenity(save, title.as_deref(), initial_dir.as_deref()) {
        PickerResult::Selected(path) => return Some(path),
        PickerResult::BackendAvailableNoSelection => backend_available = true,
        PickerResult::BackendUnavailable => {}
    }

    match pick_with_kdialog(save, title.as_deref(), initial_dir.as_deref()) {
        PickerResult::Selected(path) => return Some(path),
        PickerResult::BackendAvailableNoSelection => backend_available = true,
        PickerResult::BackendUnavailable => {}
    }

    let _ = backend_available;
    None
}

#[cfg(target_pointer_width = "32")]
enum PickerResult {
    Selected(PathBuf),
    BackendAvailableNoSelection,
    BackendUnavailable,
}

#[cfg(target_pointer_width = "32")]
fn pick_with_zenity(save: bool, title: Option<&str>, initial_dir: Option<&str>) -> PickerResult {
    let mut cmd = Command::new("zenity");
    cmd.arg("--file-selection");

    if save {
        cmd.arg("--save").arg("--confirm-overwrite");
    }

    if let Some(title) = title
        && !title.is_empty()
    {
        cmd.arg("--title").arg(title);
    }

    if let Some(dir) = initial_dir
        && !dir.is_empty()
    {
        cmd.arg("--filename").arg(format!("{dir}/"));
    }

    let output = match cmd.output() {
        Ok(output) => output,
        Err(_) => return PickerResult::BackendUnavailable,
    };

    if !output.status.success() {
        return PickerResult::BackendAvailableNoSelection;
    }

    let selected = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if selected.is_empty() {
        return PickerResult::BackendAvailableNoSelection;
    }

    PickerResult::Selected(PathBuf::from(selected))
}

#[cfg(target_pointer_width = "32")]
fn pick_with_kdialog(save: bool, title: Option<&str>, initial_dir: Option<&str>) -> PickerResult {
    let mut cmd = Command::new("kdialog");

    if save {
        cmd.arg("--getsavefilename");
    } else {
        cmd.arg("--getopenfilename");
    }

    cmd.arg(initial_dir.unwrap_or("."));

    if let Some(title) = title
        && !title.is_empty()
    {
        cmd.arg("--title").arg(title);
    }

    let output = match cmd.output() {
        Ok(output) => output,
        Err(_) => return PickerResult::BackendUnavailable,
    };

    if !output.status.success() {
        return PickerResult::BackendAvailableNoSelection;
    }

    let selected = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if selected.is_empty() {
        return PickerResult::BackendAvailableNoSelection;
    }

    PickerResult::Selected(PathBuf::from(selected))
}
