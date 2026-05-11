use rine_types::handles::HINSTANCE;
use rine_types::os::get_version;
use rine_types::windows::HWND;
use rine_types::{errors::BOOL, handles::HANDLE};

use tracing::{info, warn};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShellAboutTextFormatting {
    LegacyNt5,
    VistaAndNewer,
}

#[derive(Debug, Clone)]
struct ShellAboutTextLayout {
    title: String,
    microsoft_line: Option<String>,
    other_stuff: Option<String>,
}

fn shell_about_text_formatting() -> ShellAboutTextFormatting {
    let version = get_version();
    if version.major < 6 {
        ShellAboutTextFormatting::LegacyNt5
    } else {
        ShellAboutTextFormatting::VistaAndNewer
    }
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

fn split_title_and_microsoft_line(app_text: &str) -> (String, Option<String>) {
    if let Some((title, microsoft_line)) = app_text.split_once('#') {
        return (title.to_string(), Some(microsoft_line.to_string()));
    }

    (app_text.to_string(), None)
}

fn build_shell_about_layout(
    formatting: ShellAboutTextFormatting,
    app_text: &str,
    other_stuff: Option<&str>,
) -> ShellAboutTextLayout {
    let app_text = match formatting {
        // Vista and newer cap this input at 200 characters.
        ShellAboutTextFormatting::VistaAndNewer => truncate_chars(app_text, 200),
        ShellAboutTextFormatting::LegacyNt5 => app_text.to_string(),
    };

    let (title, explicit_microsoft_line) = split_title_and_microsoft_line(&app_text);
    let microsoft_line = match formatting {
        // Windows 2000/XP/2003 repeat the app text after "Microsoft" when '#' is absent.
        ShellAboutTextFormatting::LegacyNt5 => {
            if explicit_microsoft_line.is_some() {
                explicit_microsoft_line
            } else {
                Some(app_text)
            }
        }
        // Vista/Server 2008+ only show a replacement line when '#' is present.
        ShellAboutTextFormatting::VistaAndNewer => explicit_microsoft_line,
    };

    ShellAboutTextLayout {
        title,
        microsoft_line,
        other_stuff: other_stuff.map(str::to_string),
    }
}

/// Displays a ShellAbout dialog box (ANSI variant).
///
/// # Arguments
/// * `_hwnd` - Parent window handle. Can be `HWND::NULL`.
/// * `app_text` - Application text used for title and Microsoft line formatting.
/// * `other_stuff` - Optional extra text shown in the lower dialog body.
/// * `_icon` - Optional icon handle.
///
/// # Return
/// Returns `1` on success and `0` on failure.
///
/// # Notes
/// This implementation applies the documented text-layout split between
/// Windows 2000/XP/Server 2003 and Windows Vista/Server 2008+.
pub fn shell_about(
    _hwnd: HWND,
    app_text: Option<&str>,
    other_stuff: Option<&str>,
    _icon: HANDLE,
) -> BOOL {
    let Some(app_text) = app_text else {
        warn!("ShellAboutA failed: szApp is NULL");
        return BOOL::FALSE;
    };

    let formatting = shell_about_text_formatting();
    let layout = build_shell_about_layout(formatting, app_text, other_stuff);

    // UI hosting for this dialog is not wired yet; we still resolve layout semantics now.
    info!(
        ?formatting,
        title = %layout.title,
        microsoft_line = layout.microsoft_line.as_deref().unwrap_or(""),
        other_stuff = layout.other_stuff.as_deref().unwrap_or(""),
        "ShellAboutA layout resolved"
    );

    BOOL::TRUE
}

/// Executes a shell operation on a file or object.
///
/// # Arguments
/// * `_hwnd` - Optional owner window handle.
/// * operation - Optional operation verb (for example, "open" or "print").
/// * file - Target file/object path.
/// * parameters - Optional command-line parameters.
/// * directory - Optional working directory.
/// * show_cmd - Window show command.
///
/// # Return
/// Returns an `HINSTANCE`-typed result where values `<= 32` represent failure.
///
/// # Notes
/// This is currently a stub and does not launch processes. It reports
/// arguments for diagnostics and returns a failure code placeholder.
pub fn shell_execute(
    _hwnd: HWND,
    operation: Option<&str>,
    file: Option<&str>,
    parameters: Option<&str>,
    directory: Option<&str>,
    show_cmd: i32,
) -> HINSTANCE {
    let Some(file) = file else {
        warn!("ShellExecute failed: lpFile is NULL");
        return HINSTANCE::from_raw(0);
    };

    warn!(
        operation = operation.unwrap_or(""),
        file,
        parameters = parameters.unwrap_or(""),
        directory = directory.unwrap_or(""),
        show_cmd,
        "ShellExecute stub called"
    );

    // 31 (SE_ERR_NOASSOC) is a common ShellExecute failure code.
    HINSTANCE::from_raw(31)
}

#[cfg(test)]
mod tests {
    use super::{ShellAboutTextFormatting, build_shell_about_layout};

    #[test]
    fn shell_about_legacy_repeats_app_text_without_separator() {
        let layout = build_shell_about_layout(
            ShellAboutTextFormatting::LegacyNt5,
            "My App",
            Some("Build 42"),
        );

        assert_eq!(layout.title, "My App");
        assert_eq!(layout.microsoft_line.as_deref(), Some("My App"));
        assert_eq!(layout.other_stuff.as_deref(), Some("Build 42"));
    }

    #[test]
    fn shell_about_modern_hides_app_text_without_separator() {
        let layout =
            build_shell_about_layout(ShellAboutTextFormatting::VistaAndNewer, "My App", None);

        assert_eq!(layout.title, "My App");
        assert_eq!(layout.microsoft_line, None);
    }

    #[test]
    fn shell_about_separator_replaces_microsoft_line() {
        let legacy =
            build_shell_about_layout(ShellAboutTextFormatting::LegacyNt5, "Title#Line", None);
        let modern =
            build_shell_about_layout(ShellAboutTextFormatting::VistaAndNewer, "Title#Line", None);

        assert_eq!(legacy.title, "Title");
        assert_eq!(legacy.microsoft_line.as_deref(), Some("Line"));
        assert_eq!(modern.title, "Title");
        assert_eq!(modern.microsoft_line.as_deref(), Some("Line"));
    }

    #[test]
    fn shell_about_modern_truncates_app_text_to_200_chars() {
        let source = "A".repeat(210);
        let layout =
            build_shell_about_layout(ShellAboutTextFormatting::VistaAndNewer, &source, None);

        assert_eq!(layout.title.len(), 200);
    }
}
