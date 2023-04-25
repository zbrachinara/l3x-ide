use egui::epaint::ahash::HashMap;
use macroquad::prelude::*;

pub struct Registers(HashMap<u64, u64>); 

pub struct Traveler {
    value: Registers, // TODO new number type representing registers directly
    position: UVec2,
}