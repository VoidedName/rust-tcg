use crate::game::GameRunState;
use crate::menus::pause_menu::PauseMenu;
use crate::RunState;
use rand::distributions::Uniform;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use strum::{EnumCount, FromRepr};
use wasm_bindgen::prelude::wasm_bindgen;

const MAX_LAYERS: usize = 9;
const MIN_LAYERS: usize = 7;
const MAX_NODES_IN_LAYER: usize = 4;
const MIN_NODES_IN_LAYER: usize = 2;

#[wasm_bindgen(typescript_custom_section)]
const MAP_EDGES: &'static str = r#"
export type MapEdges = Map<number, number[]>;
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

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct GameLevel {
    pub nodes: Vec<MapNode>,
    pub edges: HashMap<usize, HashSet<usize>>,
    pub current: usize,
    pub visited: HashSet<usize>,
}

impl GameLevel {
    pub fn new_from_random<R: Rng>(r: &mut R) -> Self {
        type N = (usize, MapNode);
        let mut edges = HashMap::new();

        let nr_layers = r.gen_range(MIN_LAYERS..=MAX_LAYERS);
        let mut nodes: Vec<Vec<N>> = vec![vec![]; nr_layers];
        nodes[0] = vec![(0, MapNode::Start)];
        let mut id = 1;

        for l in 1..nr_layers - 1 {
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
                let edges = if let Some(edges) = edges.get_mut(&previous[previous_node].0) {
                    edges
                } else {
                    edges.insert(previous[previous_node].0, HashSet::new());
                    edges.get_mut(&previous[previous_node].0).unwrap()
                };

                edges.insert(current[current_node].0);

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
                        0 => previous_node += 1,
                        1 => current_node += 1,
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
            visited: HashSet::default(),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum GameMapAction {
    Waiting,
    PauseGame,
    GoToNode(usize),
}

// TODO horribly inefficient
pub fn handle_game_map(state: &mut GameRunState) -> RunState {
    if let GameRunState::ShowingMap(map) = state {
        let render_result = crate::render_game_map(
            map.borrow().level.nodes.iter().map(|x| *x as u8).collect(),
            serde_wasm_bindgen::to_value(&map.borrow().level.edges).unwrap(),
            map.borrow().level.current,
            map.borrow().level.visited.clone().into_iter().collect(),
        );

        if let Ok(data) = render_result {
            match serde_wasm_bindgen::from_value::<GameMapAction>(data).unwrap() {
                GameMapAction::PauseGame => {
                    RunState::ShowingPauseMenu(state.clone(), PauseMenu::default())
                }
                GameMapAction::GoToNode(node) => {
                    let can_path = map
                        .borrow()
                        .level
                        .edges
                        .get(&map.borrow().level.current)
                        .map_or(false, |x| x.contains(&node));
                    if can_path {
                        let mut mut_map = map.borrow_mut();
                        let current = mut_map.level.current;
                        mut_map.level.visited.insert(current);
                        mut_map.level.current = node;
                        RunState::PlayingGame(GameRunState::ShowingMap(map.clone()))
                    } else {
                        RunState::PlayingGame(state.clone())
                    }
                }
                GameMapAction::Waiting => RunState::PlayingGame(state.clone()),
            }
        } else {
            RunState::PlayingGame(state.clone())
        }
    } else {
        RunState::PlayingGame(state.clone())
    }
}
