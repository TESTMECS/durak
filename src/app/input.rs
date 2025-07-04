use crate::app::state::AppState;
use crate::game::GamePhase;
use crossterm::event::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    // General Actions
    Quit,
    ToggleDebug,
    // Main Menu Actions
    StartGame,
    ShowRules,
    ShowDifficultySelect,
    SelectEasyDifficulty,
    SelectMediumDifficulty,
    SelectHardDifficulty,
    // Rules Page Actions
    ReturnToMenu,
    // Playing Actions
    SelectNextCard,
    SelectPrevCard,
    ToggleMultiSelect,
    ToggleCardSelection,
    PlaySelectedCard, // Covers both single and multi-select Enter press
    PassTurn,         // Covers 'p' key
    TakeCards,        // Covers 't' key
    // Game Over Actions
    StartNewGame,
    // Drawing Phase Actions
    AcknowledgeDraw, // Any key during drawing
}
/// Handle User Input depending on the current AppState and GamePhase
pub fn handle_key_input(
    app_state: &AppState,
    game_phase: &GamePhase,
    key: KeyCode,
) -> Option<AppAction> {
    // Handle global keys first
    match key {
        KeyCode::Char('q') | KeyCode::Char('Q') => return Some(AppAction::Quit),
        _ => {}
    }
    match app_state {
        AppState::MainMenu => match key {
            KeyCode::Char('s') | KeyCode::Char('S') => Some(AppAction::StartGame),
            KeyCode::Char('r') | KeyCode::Char('R') => Some(AppAction::ShowRules),
            KeyCode::Char('a') | KeyCode::Char('A') => Some(AppAction::ShowDifficultySelect),
            KeyCode::Char('d') | KeyCode::Char('D') => Some(AppAction::ToggleDebug),
            _ => None,
        },
        AppState::DifficultySelect => match key {
            KeyCode::Char('1') => Some(AppAction::SelectEasyDifficulty),
            KeyCode::Char('2') => Some(AppAction::SelectMediumDifficulty),
            KeyCode::Char('3') => Some(AppAction::SelectHardDifficulty),
            KeyCode::Char('b') | KeyCode::Char('B') | KeyCode::Esc => Some(AppAction::ReturnToMenu),
            _ => None,
        },
        AppState::RulesPage => match key {
            KeyCode::Char('b') | KeyCode::Char('B') | KeyCode::Esc => Some(AppAction::ReturnToMenu),
            _ => None,
        },
        AppState::Playing => {
            match game_phase {
                GamePhase::Drawing => {
                    // Any key press acknowledges the draw phase
                    Some(AppAction::AcknowledgeDraw)
                }
                GamePhase::Attack | GamePhase::Defense => match key {
                    // vim keys
                    KeyCode::Char('k') | KeyCode::Char('h') => Some(AppAction::SelectPrevCard),
                    KeyCode::Char('j') | KeyCode::Char('l') => Some(AppAction::SelectNextCard),
                    KeyCode::Up | KeyCode::Left => Some(AppAction::SelectPrevCard),
                    KeyCode::Down | KeyCode::Right => Some(AppAction::SelectNextCard),
                    KeyCode::Char('d') | KeyCode::Char('D') => Some(AppAction::ToggleDebug),
                    KeyCode::Char('m') | KeyCode::Char('M') => Some(AppAction::ToggleMultiSelect),
                    KeyCode::Char(' ') => Some(AppAction::ToggleCardSelection),
                    KeyCode::Enter => Some(AppAction::PlaySelectedCard),
                    KeyCode::Char('p') | KeyCode::Char('P') if *game_phase == GamePhase::Attack => {
                        Some(AppAction::PassTurn)
                    }
                    KeyCode::Char('t') | KeyCode::Char('T')
                        if *game_phase == GamePhase::Defense =>
                    {
                        Some(AppAction::TakeCards)
                    }
                    _ => None,
                },
                GamePhase::GameOver => match key {
                    KeyCode::Char('n') | KeyCode::Char('N') => Some(AppAction::StartNewGame),
                    _ => None,
                },
                _ => None, // Setup phase has no input
            }
        }
        AppState::GameOver => match key {
            KeyCode::Char('n') | KeyCode::Char('N') => Some(AppAction::StartNewGame),
            _ => None,
        },
    }
}
