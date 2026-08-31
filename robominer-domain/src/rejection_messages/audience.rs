//! Player-facing vs CLI copy for shared rejection messages.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Audience {
    Player,
    Cli,
}
