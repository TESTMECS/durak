use crate::app::state::AppState; // Import AppState
use crate::app::App; // Import App from the app module
use crate::ui::debug_overlay::DebugOverlay;
use crate::ui::game_ui::GameUI;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
/// Renders the UI for the game based on the matching AppState.
pub fn render_ui(app: &App, f: &mut Frame<'_>) {
    let area = f.size();
    match app.app_state {
        AppState::MainMenu => {
            let title = Paragraph::new("Durak Card Game")
                .style(Style::default().fg(Color::Green))
                .alignment(ratatui::layout::Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            let difficulty_text = format!("Current AI Difficulty: {}", app.selected_difficulty);
            let menu = Paragraph::new(vec![
                Line::from("Press 's' to start a new game"),
                Line::from("Press 'a' to change AI difficulty"),
                Line::from("Press 'r' to view game rules"),
                Line::from("Press 'q' to quit"),
                Line::from("Press 'd' to toggle debug overlay"),
                Line::from(""),
                Line::from(difficulty_text),
            ])
            .style(Style::default().fg(Color::White))
            .alignment(ratatui::layout::Alignment::Center);
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(40),
                    Constraint::Length(3),
                    Constraint::Length(8),
                    Constraint::Percentage(40),
                ])
                .split(area);
            f.render_widget(title, layout[1]);
            f.render_widget(menu, layout[2]);
        }
        AppState::DifficultySelect => {
            // Render difficulty selection screen
            let title = Paragraph::new("Select AI Difficulty")
                .style(Style::default().fg(Color::Green))
                .alignment(ratatui::layout::Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            let current_difficulty = format!("Current: {}", app.selected_difficulty);
            let menu = Paragraph::new(vec![
                Line::from("Press '1' for Easy AI"),
                Line::from("Press '2' for Medium AI"),
                Line::from("Press '3' for Hard AI"),
                Line::from(""),
                Line::from(current_difficulty),
                Line::from(""),
                Line::from("Press 'b' to go back to main menu"),
            ])
            .style(Style::default().fg(Color::White))
            .alignment(ratatui::layout::Alignment::Center);
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(40),
                    Constraint::Length(3),
                    Constraint::Length(8),
                    Constraint::Percentage(40),
                ])
                .split(area);
            f.render_widget(title, layout[1]);
            f.render_widget(menu, layout[2]);
        }
        AppState::RulesPage => {
            // Render rules page
            let title = Paragraph::new("Durak Game Rules")
                .style(Style::default().fg(Color::Green))
                .alignment(ratatui::layout::Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            let rules_text = vec![
                Line::from(""),
                Line::from(" "),
                Line::from("Objective:"),
                Line::from("Get rid of all your cards. The last player with cards is the 'durak' (fool)."),
                Line::from(" "),
                Line::from("Setup:"),
                Line::from("- Each player gets 6 cards from a 36-card deck (6 to Ace)"),
                Line::from("- Bottom card determines trump suit (higher priority)"),
                Line::from("- Player with lowest trump card goes first"),
                Line::from(" "),
                Line::from("Gameplay:"),
                Line::from("- Attacker plays a card; defender must beat it with higher card of same suit or trump"),
                Line::from("- Passing: Defender can PASS a card by playing same rank (7♠ → 7♥) to the next player"),
                Line::from("- When a pass occurs, the original attacker must now defend against both cards"),
                Line::from("- After successful defense, attacker can add cards of the same rank as those on table"),
                Line::from("- Defender can defend against multiple cards if they have matching cards"),
                Line::from("- If defender can't or won't defend, they pick up all cards on the table"),
                Line::from("- After successful defense, defender becomes next attacker"),
                Line::from("- Players draw after each round to maintain 6 cards (attacker draws first)"),
                Line::from(" "),
                Line::from("Multiple Card Attacks:"),
                Line::from("- Press 'm' to toggle multiple selection mode"),
                Line::from("- Use spacebar to select/deselect multiple cards with the same rank"),
                Line::from("- Press Enter to play all selected cards at once"),
                Line::from("- You can only attack with cards of ranks already on the table"),
                Line::from("- You cannot attack with more cards than the defender has in hand"),
                Line::from(" "),
                Line::from("End Game:"),
                Line::from("- Once deck is empty and a player has no cards left, that player is out"),
                Line::from("- The last player with cards is the 'durak'"),
                Line::from(" "),
                Line::from("Press 'b' to go back to the main menu"),
            ];
            let rules = Paragraph::new(rules_text)
                .style(Style::default().fg(Color::White))
                .block(Block::default().borders(Borders::ALL).title("Game Rules"));
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(10)])
                .split(area);
            f.render_widget(title, layout[0]);
            f.render_widget(rules, layout[1]);
        }
        AppState::Playing => {
            let mut game_ui = GameUI::new(&app.game_state).select_card(app.selected_card_idx);
            if app.multiple_selection_mode {
                game_ui = game_ui.with_multiple_selection(&app.selected_cards);
            }
            f.render_widget(game_ui, area);
        }
        AppState::GameOver => {
            // Create the winner message
            let winner_message = if let Some(winner_idx) = app.game_state.winner() {
                let winner_name = &app.game_state.players()[winner_idx].name();
                format!("{} is the winner!", winner_name)
            } else {
                "Game Over!".to_string()
            };
            // Layout for the game over screen
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Length(3), // Title
                    Constraint::Length(3), // Winner message
                    Constraint::Length(3), // Instructions
                    Constraint::Percentage(30),
                ])
                .split(area);
            // Game over title
            let title = Paragraph::new("Game Over")
                .style(Style::default().fg(Color::Green))
                .alignment(ratatui::layout::Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            // Winner message
            let winner_text = Paragraph::new(winner_message)
                .style(Style::default().fg(Color::Yellow))
                .alignment(ratatui::layout::Alignment::Center);
            // Instructions
            let instructions = Paragraph::new("Press 'N' for new game | Press 'Q' to quit")
                .style(Style::default().fg(Color::White))
                .alignment(ratatui::layout::Alignment::Center);
            // Render all components
            f.render_widget(title, layout[1]);
            f.render_widget(winner_text, layout[2]);
            f.render_widget(instructions, layout[3]);
        }
    }
    if app.show_debug {
        let debug_overlay = DebugOverlay::new();
        f.render_widget(debug_overlay, area);
    }
}
