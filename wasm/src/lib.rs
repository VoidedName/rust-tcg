use std::cell::RefCell;
use std::rc::Rc;
use rand::SeedableRng;
use rand_xoshiro::Xoroshiro128PlusPlus;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use menus::{main_menu, MenuAction, settings_menu};
use menus::pause_menu::PauseMenu;
use crate::menus::main_menu::MainMenu;
use crate::menus::pause_menu;
use crate::menus::settings_menu::SettingsMenu;

mod menus;

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// ```plantuml
// @startuml
// state Initializing
// state StartingNewGame
// state ShowingMainMenu
// state ShowingSettingsMenu
// state LoadingSavedGame
// state Quitting
// state PlayingGame
// state PausingGame
//
// [*] --> Initializing
//
// Initializing --> ShowingMainMenu
//
// state ShowingMainMenu {
//   state SelectedPlay
//   state SelectedContinue
//   state SelectedSettings
//   state SelectedQuit
//
//   state can_continue_down <<choice>>
//   state can_continue_up <<choice>>
//
//   [*] --> SelectedPlay
//   SelectedPlay --> can_continue_down : OnNext
//   can_continue_down --> SelectedContinue : [has save file]
//   can_continue_down --> SelectedSettings : [no save file]
//   SelectedContinue --> SelectedSettings : OnNext
//   SelectedSettings --> SelectedQuit : OnNext
//   SelectedQuit --> SelectedQuit : OnNext
//
//   SelectedQuit --> SelectedSettings : OnPrevious
//   SelectedSettings --> can_continue_up : OnPrevious
//   can_continue_up --> SelectedContinue : [has save file]
//   can_continue_up --> SelectedPlay : [no save file]
//   SelectedContinue --> SelectedPlay : OnPrevious
//   SelectedPlay --> SelectedPlay : OnPrevious
//
//
//   SelectedPlay --> StartingNewGame : OnConfirm
//   SelectedSettings --> ShowingSettingsMenu : OnConfirm
//   SelectedContinue --> LoadingSavedGame : OnConfirm
//   SelectedQuit --> Quitting: OnConfirm
// }
//
// state ShowingSettingsMenu {
//   [*] --> SelectedBack
//   SelectedBack --> SelectedSettings : OnConfirm
// }
//
// state PlayingGame {
//   state ShowingMap
//   state ShowingEncounter
//
//   ShowingMap --> ShowingEncounter : OnStartEncounter
//   ShowingEncounter --> ShowingMap : OnEncounterEnd
//
//   PlayingGame --> PausingGame : OnPause
//
//   ShowingMap : MapState
//   ShowingEncounter : MapState
//   ShowingEncounter : EncounterState
// }
//
// state PausingGame {
//   [*] --> ContinuePausedGame
//
//   ContinuePausedGame --> PlayingGame : OnConfirm
//   SaveAndQuitGame --> ShowingMainMenu: OnConfirm
//
//   SaveAndQuitGame --> SaveAndQuitGame : OnNext
//   ContinuePausedGame --> SaveAndQuitGame : OnNext
//
//   ContinuePausedGame --> ContinuePausedGame : OnPrevious
//   SaveAndQuitGame --> ContinuePausedGame : OnPrevious
// }
//
// StartingNewGame --> PlayingGame : OnGameGenerated
// LoadingSavedGame --> PlayingGame : OnGameLoaded
//
// Quitting --> [*]
// @enduml
// ```
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum RunState {
    Initializing,
    ShowingMainMenu(MainMenu),
    StartingNewGame,
    LoadingSavedGame,
    ShowingSettingsMenu(SettingsMenu),
    Quitting,
    PlayingGame(GameRunState),
    PausingGame(GameRunState, PauseMenu),
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct GameState {
    r: Xoroshiro128PlusPlus,
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct EncounterState {}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum GameRunState {
    ShowingMap(Rc<RefCell<GameState>>),
    ShowingEncounter(Rc<RefCell<GameState>>, EncounterState),
}

#[wasm_bindgen]
pub enum GameMapActions {
    PauseGame,
}

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let ticker = Rc::new(RefCell::new(None));
    let tmp_ticker = ticker.clone();

    let mut start = None;
    let run_state = Rc::new(RefCell::new(RunState::Initializing));
    let mut save_game = None;

    *tmp_ticker.borrow_mut() = Some(Closure::new(move |now: f64| {
        let delta;
        if let Some(start_value) = start {
            delta = now - start_value;
            start = Some(now);
        } else {
            start = Some(now);
            delta = 0.0;
        }

        println!("{:?}", delta);

        /////////////////////////
        // GAME LOOP GOES HERE //
        /////////////////////////

        let next_state = match &*run_state.borrow() {
            RunState::Initializing => {
                save_game = get_save_game().unwrap();
                RunState::ShowingMainMenu(MainMenu::default())
            },

            RunState::ShowingMainMenu(menu) => main_menu::handle_main_menu(*menu, save_game != None),
            RunState::ShowingSettingsMenu(menu) => settings_menu::handle_settings_menu(*menu),

            RunState::StartingNewGame => RunState::PlayingGame(
                GameRunState::ShowingMap(Rc::new(RefCell::new(GameState { r: Xoroshiro128PlusPlus::from_entropy() })))
            ),
            RunState::LoadingSavedGame => {
                match &save_game {
                    Some(data) => RunState::PlayingGame(serde_json::from_str(data.as_str()).unwrap()),
                    None => RunState::Initializing,
                }
            },

            RunState::PlayingGame(state) => {
                match state {
                    GameRunState::ShowingMap(_) => handle_game_map(state.clone()),
                    GameRunState::ShowingEncounter(_, _) => run_state.borrow().clone(),
                }
            }
            RunState::PausingGame(from, menu) => pause_menu::handle_pause_menu(*menu, from.clone()),

            // just do nothing, loop will end on its own
            RunState::Quitting => run_state.borrow().clone()
        };
        *run_state.borrow_mut() = next_state;

        web_sys::console::log_1(&format!("{:?}", run_state.borrow()).into());

        if *run_state.borrow() == RunState::Quitting {
            quit_application().unwrap();
        } else {
            request_animation_frame(ticker.borrow().as_ref().unwrap()).expect("Failed to setup animation");
        }
    }));

    request_animation_frame(tmp_ticker.borrow().as_ref().unwrap()).expect("Failed to setup animation");
    Ok(())
}

fn handle_game_map(state: GameRunState) -> RunState {
    if let Ok(Some(action)) = render_game_map() {
        match action {
            GameMapActions::PauseGame => RunState::PausingGame(state, PauseMenu::default())
        }
    } else {
        RunState::PlayingGame(state)
    }
}

#[wasm_bindgen(raw_module = "../src/main")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn render_main_menu(current: u8, has_save_game: bool) -> Result<Option<MenuAction>, JsValue>;
    #[wasm_bindgen(catch)]
    fn render_settings_menu(current: u8) -> Result<Option<MenuAction>, JsValue>;
    #[wasm_bindgen(catch)]
    fn render_pause_menu(current: u8) -> Result<Option<MenuAction>, JsValue>;
    #[wasm_bindgen(catch)]
    fn render_game_map() -> Result<Option<GameMapActions>, JsValue>;
    #[wasm_bindgen(catch)]
    fn quit_application() -> Result<(), JsValue>;
    #[wasm_bindgen(catch)]
    fn save_game(data: &str) -> Result<(), JsValue>;
    #[wasm_bindgen(catch)]
    fn get_save_game() -> Result<Option<String>, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_name = requestAnimationFrame)]
    fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) -> Result<i32, JsValue>;
    #[wasm_bindgen(catch, js_name = cancelAnimationFrame)]
    fn cancel_animation_frame(handle: i32) -> Result<(), JsValue>;
}

