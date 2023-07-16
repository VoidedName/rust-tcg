use std::cell::RefCell;
use std::rc::Rc;
use rand::{Rng, SeedableRng};
use rand::distributions::Uniform;
use rand_xoshiro::Xoroshiro128PlusPlus;
use serde::{Deserialize, Serialize};
use strum::{EnumCount, FromRepr};
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
pub struct GameLevel {
    nodes: Vec<MapNode>,
    edges: Vec<MapEdge>,
    current: usize,
}

const MAX_LAYERS: usize = 9;
const MIN_LAYERS: usize = 7;
const MAX_NODES_IN_LAYER: usize = 4;
const MIN_NODES_IN_LAYER: usize = 2;

impl GameLevel {
    pub fn new_from_random<R: Rng>(r: &mut R) -> Self {
        type N = (usize, MapNode);
        let mut edges = vec![];

        let nr_layers = r.gen_range(MIN_LAYERS..=MAX_LAYERS);
        let mut nodes: Vec<Vec<N>> = vec![vec![]; nr_layers];
        nodes[0] = vec![(0, MapNode::Start)];
        let mut id = 1;

        for l in 1..nr_layers-1 {
            let layer = &mut nodes[l];
            let nr_nodes = r.gen_range(MIN_NODES_IN_LAYER..=MAX_NODES_IN_LAYER);
            for _ in 0..nr_nodes {
                // ignore first and last value in enum, as they are start and end
                // and can not show up in inner layers
                let t = (r.gen_range(0..MapNode::COUNT - 2) + 1) as u8;
                let t = MapNode::from_repr(t).unwrap();
                layer.push((id, t));
                id += 1;
            }
        }

        nodes[nr_layers - 1] = vec![(id, MapNode::End)];

        for current in (1..nr_layers).rev() {
            let previous = &nodes[current - 1];
            let current = &nodes[current];

            let mut current_node = 0;
            let mut previous_node = 0;

            loop {
                edges.push(MapEdge(previous[previous_node].0, current[current_node].0));

                let current_is_last = current_node == current.len() - 1;
                let previous_is_last = previous_node == previous.len() - 1;

                if current_is_last && previous_is_last {
                    break;
                } else if current_is_last {
                    previous_node += 1;
                } else if previous_is_last {
                    current_node += 1;
                } else {
                    match r.sample(Uniform::new(0, 2)) {
                        0 => { previous_node += 1 }
                        1 => { current_node += 1 }
                        _ => {
                            previous_node += 1;
                            current_node += 1;
                        }
                    }
                }
            }
        }

        Self {
            nodes: nodes.iter().flatten().map(|x| x.1).collect(),
            edges,
            current: 0,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct GameState {
    r: Xoroshiro128PlusPlus,
    level: GameLevel,
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct EncounterState {}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum GameRunState {
    ShowingMap(Rc<RefCell<GameState>>),
    ShowingEncounter(Rc<RefCell<GameState>>, EncounterState),
}

impl GameState {
    fn new() -> Self {
        let mut r = Xoroshiro128PlusPlus::from_entropy();
        let level = GameLevel::new_from_random(&mut r);

        Self {
            r,
            level,
        }
    }
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
                match save_game {
                    None => RunState::ShowingMainMenu(MainMenu::default()),
                    Some(_) => RunState::ShowingMainMenu(MainMenu::Continue),
                }
            }

            RunState::ShowingMainMenu(menu) => main_menu::handle_main_menu(*menu, save_game != None),
            RunState::ShowingSettingsMenu(menu) => settings_menu::handle_settings_menu(*menu),

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
                    GameRunState::ShowingMap(_) => handle_game_map(state.clone()),
                    GameRunState::ShowingEncounter(_, _) => run_state.borrow().clone(),
                }
            }
            RunState::PausingGame(from, menu) => pause_menu::handle_pause_menu(*menu, from.clone()),

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

fn handle_game_map(state: GameRunState) -> RunState {
    if let GameRunState::ShowingMap(map) = &state {
        if let Ok(data) = render_game_map(
            map.borrow().level.nodes.iter().map(|x| *x as u8).collect(),
            map.borrow().level.edges.iter().map(serde_wasm_bindgen::to_value).map(Result::unwrap).collect(),
            map.borrow().level.current,
            vec![],
        ) {
            match serde_wasm_bindgen::from_value::<GameMapAction>(data).unwrap() {
                GameMapAction::PauseGame => RunState::PausingGame(state.clone(), PauseMenu::default()),
                _ => RunState::PlayingGame(state.clone()),
            }
        } else {
            RunState::PlayingGame(state.clone())
        }
    } else {
        RunState::PlayingGame(state.clone())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum GameMapAction {
    Waiting,
    PauseGame,
    GoToNode(usize),
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct MapEdge(usize, usize);

#[wasm_bindgen(typescript_custom_section)]
const MAP_EDGE: &'static str = r#"
export type MapEdge = [number, number];
"#;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize, EnumCount, FromRepr)]
// START and END HAVE TO BE first and last member!
pub enum MapNode {
    Start,
    Combat,
    End,
}

//TODO: write macro to generate this automatically!
#[wasm_bindgen(typescript_custom_section)]
const GAME_MAP_ACTION: &'static str = r#"
export type GameMapAction = "Waiting" | "PauseGame" | { GoToNode: number };
"#;

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

