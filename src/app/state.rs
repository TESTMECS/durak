/// AppState enum for the game
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    MainMenu,
    DifficultySelect,
    RulesPage,
    Playing,
    GameOver,
}
