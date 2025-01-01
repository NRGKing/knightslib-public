use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};

pub fn create_app_menu() -> Menu {
    return Menu::new()
        .add_submenu(Submenu::new(
            "File",
            Menu::new()
                .add_item(CustomMenuItem::new("save".to_string(), "Save VAW path").accelerator("Shift+S"))
                .add_item(CustomMenuItem::new("load".to_string(), "Load VAW path").accelerator("Shift+L"))
                .add_native_item(MenuItem::Separator)
                .add_item(CustomMenuItem::new("export".to_string(), "Export VAW path").accelerator("Shift+E"))
                .add_item(CustomMenuItem::new("new".to_string(), "New VAW path").accelerator("Shift+N"))
                .add_native_item(MenuItem::Separator)
                .add_item(CustomMenuItem::new("export_mvmt".to_string(), "Export Current Movement"))

        ))
        .add_submenu(Submenu::new(
            "Edit",
            Menu::new()
                .add_native_item(MenuItem::Copy)
                .add_native_item(MenuItem::Paste)
                .add_native_item(MenuItem::Undo)
                .add_native_item(MenuItem::Redo)
        ))
        .add_submenu(Submenu::new(
            "Configuration",
            Menu::new()
                .add_item(CustomMenuItem::new("change_field".to_string(), "Change Field Set Up").accelerator("Shift+L"))
        ));
}