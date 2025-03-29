use std::collections::HashMap;

use crate::types::TrainerCard;

// TODO: Probably best to generate this file from database.json via card_enum_generator.rs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolId {
    A2147GiantCape,
    A2148RockyHelmet,
}

lazy_static::lazy_static! {
    static ref TOOL_ID_MAP: HashMap<&'static str, ToolId> = {
        let mut m = HashMap::new();
        m.insert("A2 147", ToolId::A2147GiantCape);
        m.insert("A2 148", ToolId::A2148RockyHelmet);
        m
    };
}

impl ToolId {
    pub fn from_trainer_card(trainer_card: &TrainerCard) -> Option<&Self> {
        TOOL_ID_MAP.get(&trainer_card.id.as_str())
    }
}
