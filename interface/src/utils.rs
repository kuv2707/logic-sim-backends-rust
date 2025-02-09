use std::hash::Hash;

use bsim_engine::types::{ID, PIN};
use egui::Vec2;

#[derive(Clone, Eq, PartialEq)]
pub struct CompIO {
    pub id: ID,
    pub pin: PIN,
    pub loc_rel: Vec2,
    pub label: String,
}

impl Hash for CompIO {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.pin.hash(state);
        state.write_i64((self.loc_rel.x * 1000000.0 + self.loc_rel.y * 1000.0) as i64);
    }
}

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct EmitterReceiverPair {
    pub emitter: (egui::Id, CompIO),
    pub receiver: (egui::Id, CompIO),
}

#[macro_export]
macro_rules! true_false_color {
    ($a: expr) => {
        if $a {
            $crate::consts::GREEN_COL
        } else {
            $crate::consts::RED_COL
        }
    };
}
