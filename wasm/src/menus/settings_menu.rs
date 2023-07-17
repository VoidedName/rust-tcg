use crate::menus::main_menu::MainMenu;
use crate::menus::MenuAction;
use crate::RunState;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SettingsMenu {
    Back,
}

impl Default for SettingsMenu {
    fn default() -> Self {
        Self::Back
    }
}

impl SettingsMenu {
    pub fn on_action(self, action: MenuAction) -> RunState {
        match action {
            MenuAction::Next => RunState::ShowingSettingsMenu(self.on_next()),
            MenuAction::Previous => RunState::ShowingSettingsMenu(self.on_previous()),
            MenuAction::Confirm => self.on_confirm(),
        }
    }

    fn on_next(self) -> Self {
        Self::Back
    }

    fn on_previous(self) -> Self {
        Self::Back
    }

    fn on_confirm(self) -> RunState {
        match self {
            SettingsMenu::Back => RunState::ShowingMainMenu(MainMenu::Settings),
        }
    }
}

pub fn handle_settings_menu(current: SettingsMenu) -> RunState {
    if let Ok(Some(action)) = crate::render_settings_menu(current as u8) {
        current.on_action(action)
    } else {
        RunState::ShowingSettingsMenu(current)
    }
}
