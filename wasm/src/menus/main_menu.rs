use crate::menus::settings_menu::SettingsMenu;
use crate::menus::MenuAction;
use crate::RunState;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MainMenu {
    Play,
    Continue,
    Settings,
    Quit,
}

impl Default for MainMenu {
    fn default() -> Self {
        Self::Play
    }
}

impl MainMenu {
    pub fn on_action(self, action: MenuAction, has_save_game: bool) -> RunState {
        match action {
            MenuAction::Next => RunState::ShowingMainMenu(self.on_next(has_save_game)),
            MenuAction::Previous => RunState::ShowingMainMenu(self.on_previous(has_save_game)),
            MenuAction::Confirm => self.on_confirm(),
        }
    }

    fn on_next(self, has_save_game: bool) -> MainMenu {
        match self {
            MainMenu::Play => {
                if has_save_game {
                    MainMenu::Continue
                } else {
                    MainMenu::Settings
                }
            }
            MainMenu::Continue => MainMenu::Settings,
            MainMenu::Settings => MainMenu::Quit,
            MainMenu::Quit => MainMenu::Quit,
        }
    }

    fn on_previous(self, has_save_game: bool) -> MainMenu {
        match self {
            MainMenu::Play => MainMenu::Play,
            MainMenu::Continue => MainMenu::Play,
            MainMenu::Settings => {
                if has_save_game {
                    MainMenu::Continue
                } else {
                    MainMenu::Play
                }
            }
            MainMenu::Quit => MainMenu::Settings,
        }
    }

    fn on_confirm(self) -> RunState {
        match self {
            MainMenu::Play => RunState::StartingNewGame,
            MainMenu::Continue => RunState::LoadingSavedGame,
            MainMenu::Settings => RunState::ShowingSettingsMenu(SettingsMenu::default()),
            MainMenu::Quit => RunState::Quitting,
        }
    }
}

pub fn handle_main_menu(current: MainMenu, has_save_game: bool) -> RunState {
    if let Ok(Some(action)) = crate::render_main_menu(current as u8, has_save_game) {
        current.on_action(action, has_save_game)
    } else {
        RunState::ShowingMainMenu(current)
    }
}
