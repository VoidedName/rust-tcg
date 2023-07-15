use wasm_bindgen::prelude::wasm_bindgen;
use crate::{GameRunState, RunState, save_game};
use crate::menus::main_menu::MainMenu;
use crate::menus::MenuAction;

#[wasm_bindgen]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PauseMenu {
    Continue,
    SaveAndQuit,
}

impl Default for PauseMenu {
    fn default() -> Self {
        Self::Continue
    }
}

impl PauseMenu {
    pub fn on_action(self, action: MenuAction, game_state: GameRunState) -> RunState {
        match action {
            MenuAction::Next => RunState::PausingGame(game_state, self.on_next()),
            MenuAction::Previous => RunState::PausingGame(game_state, self.on_previous()),
            MenuAction::Confirm => self.on_confirm(game_state),
        }
    }

    fn on_next(self) -> Self {
        match self {
            PauseMenu::Continue => PauseMenu::SaveAndQuit,
            PauseMenu::SaveAndQuit => PauseMenu::SaveAndQuit,
        }
    }

    fn on_previous(self) -> Self {
        match self {
            PauseMenu::Continue => PauseMenu::Continue,
            PauseMenu::SaveAndQuit => PauseMenu::Continue,
        }
    }

    fn on_confirm(self, game_state: GameRunState) -> RunState {
        match self {
            PauseMenu::Continue => RunState::PlayingGame(game_state),
            //TODO
            PauseMenu::SaveAndQuit => {
                save_game(serde_json::to_string(&game_state).unwrap().as_str()).expect("Failed to save game");
                RunState::Initializing
            },
        }
    }
}

pub fn handle_pause_menu(current: PauseMenu, game_state: GameRunState) -> RunState {
    if let Ok(Some(action)) = crate::render_pause_menu(current as u8) {
        current.on_action(action, game_state)
    } else {
        RunState::PausingGame(game_state, current)
    }
}
