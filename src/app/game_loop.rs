/*
 * game_loop.rs - Main game loop and event handling
 *
 * This file contains the main game loop and event handling logic:
 * - Processing user input
 * - Rendering the UI
 * - Main event loop with input polling
 */
use super::app_core::App;
use super::input::{handle_key_input, AppAction};
use crate::ui::debug_overlay::{error, trace};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::backend::Backend;
use ratatui::Terminal;
use std::io;

impl App {
    /// On key input, check if there is an action mapped to the key and process it
    pub fn on_key(&mut self, key: KeyCode) {
        trace(format!(
            "Key: {:?}, State: {:?}, Phase: {:?}",
            key,
            self.app_state,
            self.game_state.game_phase(),
        ));
        if let Some(action) = handle_key_input(&self.app_state, self.game_state.game_phase(), key) {
            self.process_action(action);
        } else {
            trace("No action mapped for key");
        }
    }
    /// Process an action from the user, main logic called here.
    fn process_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit => self.quit(),
            AppAction::ToggleDebug => self.toggle_debug(),
            AppAction::ShowRules => self.show_rules(),
            AppAction::ShowDifficultySelect => self.show_difficulty_select(),
            AppAction::SelectEasyDifficulty => {
                self.select_difficulty(crate::game::AiDifficulty::Easy)
            }
            AppAction::SelectMediumDifficulty => {
                self.select_difficulty(crate::game::AiDifficulty::Medium)
            }
            AppAction::SelectHardDifficulty => {
                self.select_difficulty(crate::game::AiDifficulty::Hard)
            }
            AppAction::ReturnToMenu => self.return_to_menu(),
            AppAction::SelectNextCard => self.select_next_card(),
            AppAction::SelectPrevCard => self.select_prev_card(),
            AppAction::ToggleMultiSelect => self.toggle_multiple_selection(),
            AppAction::ToggleCardSelection => {
                if self.multiple_selection_mode {
                    if let Some(idx) = self.selected_card_idx {
                        self.toggle_card_selection(idx);
                    }
                }
            }
            AppAction::StartGame => self.start_game_action(),
            AppAction::PlaySelectedCard => self.play_card_action(),
            AppAction::PassTurn => self.pass_turn_action(),
            AppAction::TakeCards => self.take_cards_action(),
            AppAction::StartNewGame => self.start_game_action(), // Restart current game
            AppAction::AcknowledgeDraw => self.acknowledge_draw_action(),
        }
    }

    /// Main game loop
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        while !self.should_quit {
            // Handle any render errors
            if let Err(e) = self.render(terminal) {
                error(format!("Render error: {}", e));
                return self.safe_exit(Some(&format!("Render error: {}", e)));
            }
            // Read user input every 100ms
            match event::poll(std::time::Duration::from_millis(100)) {
                Ok(has_event) => {
                    if has_event {
                        match event::read() {
                            Ok(Event::Key(key)) => {
                                if key.kind == KeyEventKind::Press {
                                    // Process key input - if critical errors occur they will trigger safe_exit
                                    self.on_key(key.code);
                                }
                            }
                            Ok(_) => {} // Other events we ignore
                            Err(e) => {
                                error(format!("Event read error: {}", e));
                                return self.safe_exit(Some(&format!("Event read error: {}", e)));
                            }
                        }
                    }
                }
                Err(e) => {
                    error(format!("Event poll error: {}", e));
                    return self.safe_exit(Some(&format!("Event poll error: {}", e)));
                }
            }
        }
        Ok(())
    }
}
