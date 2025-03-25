use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::game::{GamePhase, GameState};
use super::card_view::{CardRowView, TableView};

pub struct GameUI<'a> {
    game_state: &'a GameState,
    selected_card_idx: Option<usize>,
}

impl<'a> GameUI<'a> {
    pub fn new(game_state: &'a GameState) -> Self {
        Self {
            game_state,
            selected_card_idx: None,
        }
    }
    
    pub fn select_card(mut self, idx: Option<usize>) -> Self {
        self.selected_card_idx = idx;
        self
    }
    
    fn render_status_bar(&self, area: Rect, buf: &mut Buffer) {
        let phase_text = match self.game_state.game_phase() {
            GamePhase::Setup => "Setting up game...".to_string(),
            GamePhase::Attack => {
                let attacker = &self.game_state.players()[self.game_state.current_attacker()];
                format!("{}'s turn to attack", attacker.name())
            },
            GamePhase::Defense => {
                let defender = &self.game_state.players()[self.game_state.current_defender()];
                format!("{}'s turn to defend", defender.name())
            },
            GamePhase::Drawing => "Drawing cards...".to_string(),
            GamePhase::GameOver => {
                if let Some(winner_idx) = self.game_state.winner() {
                    let winner = &self.game_state.players()[winner_idx];
                    format!("Game over! {} is the winner!", winner.name())
                } else {
                    "Game over!".to_string()
                }
            },
        };
        
        let trump_text = if let Some(trump_suit) = self.game_state.trump_suit() {
            format!("Trump: {}", trump_suit.symbol())
        } else {
            "No trump".to_string()
        };
        
        let deck_count = format!("Cards left: {}", self.game_state.deck().remaining());
        
        let status_line = Line::from(vec![
            Span::styled(phase_text, Style::default().fg(Color::Green)),
            Span::raw(" | "),
            Span::styled(trump_text, Style::default().fg(Color::Yellow)),
            Span::raw(" | "),
            Span::styled(deck_count, Style::default().fg(Color::Cyan)),
        ]);
        
        let paragraph = Paragraph::new(status_line)
            .block(Block::default().borders(Borders::ALL).title("Game Status"));
            
        paragraph.render(area, buf);
    }
    
    fn render_player_hand(&self, area: Rect, buf: &mut Buffer, player_idx: usize) {
        let player = &self.game_state.players()[player_idx];
        let player_name = player.name();
        let is_current_player = player_idx == self.game_state.current_attacker() || 
                                player_idx == self.game_state.current_defender();
        
        let title_style = if is_current_player {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };
        
        let block = Block::default()
            .borders(Borders::ALL)
            .title(player_name)
            .title_style(title_style);
        
        // Get inner area before rendering block
        let inner_area = block.inner(area);
            
        // Render the block
        block.render(area, buf);
        
        // Only show cards for human player
        if player.player_type() == &crate::game::PlayerType::Human {
            let selected = if player_idx == self.game_state.current_attacker() || 
                             player_idx == self.game_state.current_defender() {
                self.selected_card_idx
            } else {
                None
            };
            
            CardRowView::new(player.hand().to_vec())
                .select(selected)
                .render(inner_area, buf);
        } else {
            // For computer players, just show card backs or count
            let card_count = format!("{} cards", player.hand_size());
            let para = Paragraph::new(card_count)
                .style(Style::default().fg(Color::DarkGray));
            para.render(inner_area, buf);
        }
    }
    
    fn render_table(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Table");
        
        // Get inner area before rendering block
        let inner_area = block.inner(area);
            
        // Render the block
        block.render(area, buf);
        
        if !self.game_state.table_cards().is_empty() {
            TableView::new(self.game_state.table_cards().to_vec())
                .render(inner_area, buf);
        } else {
            let para = Paragraph::new("No cards on table")
                .style(Style::default().fg(Color::DarkGray));
            para.render(inner_area, buf);
        }
    }
    
    fn render_help(&self, area: Rect, buf: &mut Buffer) {
        let current_phase = self.game_state.game_phase();
        
        let help_text = match current_phase {
            GamePhase::Attack => "↑/↓: Select card | Enter: Play card | T: Take cards",
            GamePhase::Defense => "↑/↓: Select card | Enter: Play card | T: Take cards",
            GamePhase::GameOver => "Q: Quit | N: New game",
            _ => "",
        };
        
        let para = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title("Help"))
            .style(Style::default().fg(Color::White));
            
        para.render(area, buf);
    }
}

impl<'a> Widget for GameUI<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Status bar
                Constraint::Length(7),  // Player 1 (top, computer)
                Constraint::Min(10),    // Table (middle)
                Constraint::Length(7),  // Player 0 (bottom, human)
                Constraint::Length(3),  // Help
            ])
            .split(area);
        
        self.render_status_bar(vertical_layout[0], buf);
        
        // For a 2-player game
        if self.game_state.players().len() >= 2 {
            self.render_player_hand(vertical_layout[1], buf, 1); // Computer player
            self.render_table(vertical_layout[2], buf);
            self.render_player_hand(vertical_layout[3], buf, 0); // Human player
        }
        
        self.render_help(vertical_layout[4], buf);
    }
} 