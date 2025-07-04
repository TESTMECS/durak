/// AppState enum for the game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    MainMenu,
    DifficultySelect,
    RulesPage,
    Playing,
    GameOver,
}
