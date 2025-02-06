use image::Rgba;
use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};

pub fn to_player_name(rng: &mut ThreadRng, name: &str) -> String {
    format!(
        "{}#{:03}",
        name.chars().take(10).collect::<String>(),
        rng.gen_range(0..1000)
    )
}

pub fn random_minotaur_name() -> String {
    MINOTAUR_NAMES
        .choose(&mut rand::thread_rng())
        .unwrap()
        .to_string()
}

pub struct GameColors {}

impl GameColors {
    pub const HERO: Rgba<u8> = Rgba([3, 3, 255, 255]);
    pub const OTHER_HERO: Rgba<u8> = Rgba([3, 255, 3, 255]);
    pub const MINOTAUR: Rgba<u8> = Rgba([225, 203, 3, 255]);
    pub const CHASING_MINOTAUR: Rgba<u8> = Rgba([255, 15, 0, 255]);
    pub const POWER_UP: Rgba<u8> = Rgba([255, 180, 244, 255]);
}

pub const MINOTAUR_NAMES: [&'static str; 7] = [
    "Ἀστερίων",
    "Μίνως",
    "Σαρπηδών",
    "Ῥαδάμανθυς",
    "Ἀμφιτρύων",
    "Πτερέλαος",
    "Τάφος",
];
