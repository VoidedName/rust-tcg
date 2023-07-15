use wasm_bindgen::prelude::wasm_bindgen;

pub mod main_menu;
pub mod settings_menu;
pub mod pause_menu;

#[repr(u8)]
#[wasm_bindgen]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MenuAction {
    Next,
    Previous,
    Confirm,
}
