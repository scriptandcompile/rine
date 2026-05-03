mod binfmt;
mod config;
mod context_menu;
mod desktop;
mod run;
mod window_host;

pub use self::binfmt::{binfmt_status_cmd, install_binfmt_cmd, uninstall_binfmt_cmd};
pub use self::config::{show_config, show_config_dashboard};
pub use self::context_menu::{
    context_menu_status_cmd, install_context_menu_cmd, uninstall_context_menu_cmd,
};
pub use self::desktop::{desktop_status_cmd, install_desktop_cmd, uninstall_desktop_cmd};
pub use self::run::run;
