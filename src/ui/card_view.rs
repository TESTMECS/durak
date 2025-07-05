use crate::game::Card;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct CardView {
    card: Card,
    selected: bool,
}

impl CardView {
    pub fn new(card: Card) -> Self {
        Self {
            card,
            selected: false,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

impl Widget for CardView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 5 || area.height < 3 {
            return;
        }
        let color = if self.card.suit.is_red() {
            Color::Red
        } else {
            Color::White
        };
        let border_style = if self.selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        // Create card block
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);
        // Get inner area before rendering
        let inner_area = block.inner(area);
        // Render card block
        block.render(area, buf);
        // Render rank and suit at top-left
        let rank_suit = Paragraph::new(Line::from(vec![Span::styled(
            format!("{}{}", self.card.rank.symbol(), self.card.suit.symbol()),
            Style::default().fg(color),
        )]));
        rank_suit.render(inner_area, buf);
    }
}

pub struct CardRowView {
    cards: Vec<Card>,
    selected_idx: Option<usize>,
    multiple_selected: Option<Vec<usize>>,
}

impl CardRowView {
    pub fn new(cards: Vec<Card>) -> Self {
        Self {
            cards,
            selected_idx: None,
            multiple_selected: None,
        }
    }
    pub fn with_multiple_selection(mut self, selected: Vec<usize>) -> Self {
        self.multiple_selected = Some(selected);
        self
    }
    pub fn select(mut self, idx: Option<usize>) -> Self {
        self.selected_idx = idx;
        self
    }
}

impl Widget for CardRowView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 1 || area.height < 3 || self.cards.is_empty() {
            return;
        }
        let card_width = 8_u16;
        let spacing = 1_u16;
        let visible_cards =
            ((area.width as usize) / (card_width as usize + spacing as usize)).max(1);
        let cards_to_render = self.cards.len().min(visible_cards);
        let widths = std::iter::repeat(Constraint::Length(card_width))
            .take(cards_to_render)
            .collect::<Vec<_>>();
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(widths)
            .horizontal_margin((area.width - (card_width + spacing) * cards_to_render as u16) / 2);
        // Split the area into sections for each card
        let sections = layout.split(area);
        for (i, &card) in self.cards.iter().take(visible_cards).enumerate() {
            // Card can be selected in two ways:
            // 1. It's the currently selected card (cursor)
            // 2. It's in the multiple selection list
            let is_cursor_selected = self.selected_idx == Some(i);
            let is_multiple_selected = self
                .multiple_selected
                .as_ref()
                .is_some_and(|selected| selected.contains(&i));
            // Either selection method makes the card highlighted
            let is_selected = is_cursor_selected || is_multiple_selected;
            // Calculate card area with spacing (manually handle spacing)
            let mut card_area = sections[i];
            if i < visible_cards - 1 {
                card_area.width = card_area.width.saturating_sub(spacing);
            }
            CardView::new(card)
                .selected(is_selected)
                .render(card_area, buf);
        }
    }
}

pub struct TableView {
    table_cards: Vec<(Card, Option<Card>)>,
}

impl TableView {
    pub fn new(table_cards: Vec<(Card, Option<Card>)>) -> Self {
        Self { table_cards }
    }
}

impl Widget for TableView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 1 || area.height < 7 || self.table_cards.is_empty() {
            return;
        }
        let pair_width = 13_u16; // Each attack/defense pair needs space
        let spacing = 1_u16;
        let visible_pairs =
            ((area.width as usize) / (pair_width as usize + spacing as usize)).max(1);
        let pairs_to_render = self.table_cards.len().min(visible_pairs);
        let widths = std::iter::repeat(Constraint::Length(pair_width))
            .take(pairs_to_render)
            .collect::<Vec<_>>();
        let horizontal_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(widths)
            .horizontal_margin((area.width - (pair_width + spacing) * pairs_to_render as u16) / 2);
        // Split the area into sections for each pair
        let sections = horizontal_layout.split(area);
        for (i, (attack_card, defend_card)) in
            self.table_cards.iter().take(pairs_to_render).enumerate()
        {
            // For each pair, create a vertical layout for attack/defense cards
            let pair_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Length(3)]);
            // Split vertically for attack/defense
            let card_sections = pair_layout.split(sections[i]);
            CardView::new(*attack_card).render(card_sections[0], buf);
            if let Some(card) = defend_card {
                CardView::new(*card).render(card_sections[1], buf);
            }
        }
    }
}
