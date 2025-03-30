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

pub fn handle_key_input(app_state: &AppState, game_phase: &GamePhase, key: KeyCode) -> Option<AppAction> {
    // Handle global keys first
    match key {
        KeyCode::Char('q') | KeyCode::Char('Q') => return Some(AppAction::Quit),
        KeyCode::Char('d') | KeyCode::Char('D') => return Some(AppAction::ToggleDebug),
        _ => {}
    }

    match app_state {
        AppState::MainMenu => match key {
            KeyCode::Char('s') | KeyCode::Char('S') => Some(AppAction::StartGame),
            KeyCode::Char('r') | KeyCode::Char('R') => Some(AppAction::ShowRules),
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
                GamePhase::Attack | GamePhase::Defense => {
                    match key {
                        KeyCode::Up | KeyCode::Left => Some(AppAction::SelectPrevCard),
                        KeyCode::Down | KeyCode::Right => Some(AppAction::SelectNextCard),
                        KeyCode::Char('m') | KeyCode::Char('M') => Some(AppAction::ToggleMultiSelect),
                        KeyCode::Char(' ') => Some(AppAction::ToggleCardSelection),
                        KeyCode::Enter => Some(AppAction::PlaySelectedCard),
                        KeyCode::Char('p') | KeyCode::Char('P') if *game_phase == GamePhase::Attack => Some(AppAction::PassTurn),
                        KeyCode::Char('t') | KeyCode::Char('T') if *game_phase == GamePhase::Defense => Some(AppAction::TakeCards),
                        _ => None,
                    }
                }
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