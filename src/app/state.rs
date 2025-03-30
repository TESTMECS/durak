#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    MainMenu,
    RulesPage,
    Playing,
    GameOver,
} 