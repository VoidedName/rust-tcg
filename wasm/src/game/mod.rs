use map::GameLevel;
use rand::SeedableRng;
use rand_xoshiro::Xoroshiro128PlusPlus;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

pub mod map;

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum GameRunState {
    ShowingMap(Rc<RefCell<GameState>>),
    ShowingEncounter(Rc<RefCell<GameState>>, EncounterState),
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct EncounterState {}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct GameState {
    pub r: Xoroshiro128PlusPlus,
    pub level: GameLevel,
}

impl GameState {
    pub fn new() -> Self {
        let mut r = Xoroshiro128PlusPlus::from_entropy();
        let level = GameLevel::new_from_random(&mut r);

        Self { r, level }
    }
}
