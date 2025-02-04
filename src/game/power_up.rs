use strum::{Display, EnumIter};

#[derive(Debug, Clone, Copy, Display, PartialEq, EnumIter)]
pub enum PowerUp {
    Speed,
    Vision,
    Memory,
}
