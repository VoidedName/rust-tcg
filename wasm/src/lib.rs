use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use game::{GameRunState, GameState, map};
use menus::{main_menu, MenuAction, settings_menu};
use menus::pause_menu::PauseMenu;
use crate::menus::main_menu::MainMenu;
use crate::menus::pause_menu;
use crate::menus::settings_menu::SettingsMenu;

mod menus;
mod game;

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

//TODO document logic loop
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum RunState {
    Initializing,
    ShowingMainMenu(MainMenu),
    StartingNewGame,
    LoadingSavedGame,
    ShowingSettingsMenu(SettingsMenu),
    Quitting,
    PlayingGame(GameRunState),
    ShowingPauseMenu(GameRunState, PauseMenu),
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

        if delta < -1.0 {
            //nothing
        }

        /////////////////////////
        // GAME LOOP GOES HERE //
        /////////////////////////

        let next_state = match &*run_state.borrow() {
            RunState::Initializing => {
                save_game = get_save_game().unwrap();
                match save_game {
                    None => RunState::ShowingMainMenu(MainMenu::default()),
                    Some(_) => RunState::ShowingMainMenu(MainMenu::Continue),
                }
            }

            RunState::ShowingMainMenu(menu) => main_menu::handle_main_menu(*menu, save_game != None),
            RunState::ShowingSettingsMenu(menu) => settings_menu::handle_settings_menu(*menu),
            RunState::ShowingPauseMenu(from, menu) => pause_menu::handle_pause_menu(*menu, from.clone()),

            RunState::StartingNewGame => RunState::PlayingGame(
                GameRunState::ShowingMap(Rc::new(RefCell::new(GameState::new())))
            ),
            RunState::LoadingSavedGame => {
                match &save_game {
                    Some(data) => RunState::PlayingGame(serde_json::from_str(data.as_str()).unwrap()),
                    None => RunState::Initializing,
                }
            }

            RunState::PlayingGame(state) => {
                match state {
                    GameRunState::ShowingMap(_) => map::handle_game_map(state.clone()),
                    GameRunState::ShowingEncounter(_, _) => run_state.borrow().clone(),
                }
            }

            // just do nothing, loop will end on its own
            RunState::Quitting => run_state.borrow().clone()
        };
        *run_state.borrow_mut() = next_state;

        // web_sys::console::log_1(&format!("{:?}", run_state.borrow()).into());

        if *run_state.borrow() == RunState::Quitting {
            quit_application().unwrap();
        } else {
            request_animation_frame(ticker.borrow().as_ref().unwrap()).expect("Failed to setup animation");
        }
    }));

    request_animation_frame(tmp_ticker.borrow().as_ref().unwrap()).expect("Failed to setup animation");
    Ok(())
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
    // Result = GameMapAction
    // nodes = MapNode
    // edges = MapEdge(idx, idx)
    // visited = idx
    fn render_game_map(nodes: Vec<u8>, edges: Vec<JsValue>, current: usize, visited: Vec<usize>) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    fn quit_application() -> Result<(), JsValue>;

    #[wasm_bindgen(catch)]
    fn save_game(data: &str) -> Result<(), JsValue>;
    #[wasm_bindgen(catch)]
    fn get_save_game() -> Result<Option<String>, JsValue>;
    #[wasm_bindgen(catch)]
    fn delete_save_game() -> Result<(), JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_name = requestAnimationFrame)]
    fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) -> Result<i32, JsValue>;
    #[wasm_bindgen(catch, js_name = cancelAnimationFrame)]
    fn cancel_animation_frame(handle: i32) -> Result<(), JsValue>;
}

