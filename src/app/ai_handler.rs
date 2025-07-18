use super::app_core::App;
use crate::game::{GamePhase, PlayerType};
use crate::ui::debug_overlay::debug;

pub fn process_ai_turn(app: &mut App) {
    let mut turn_counter = 0;
    const MAX_TURNS: i32 = 10;
    while turn_counter < MAX_TURNS {
        turn_counter += 1;
        debug(format!("AI turn iteration {}", turn_counter));
        // Check for game over - this also sets the winner
        if app.game_state.check_game_over() {
            app.app_state = super::state::AppState::GameOver;
            return;
        }
        // Get the current player based on game phase
        let current_player_idx = app.current_player_index();
        let is_ai_turn =
            app.game_state.players()[current_player_idx].player_type() == &PlayerType::Computer;
        if !is_ai_turn {
            debug("Not AI's turn, ending AI processing");
            return;
        }
        debug(format!(
            "AI playing in phase: {:?}",
            app.game_state.game_phase()
        ));
        debug(format!(
            "Current attacker: {}, Current defender: {}",
            app.game_state.current_attacker(),
            app.game_state.current_defender()
        ));
        match *app.game_state.game_phase() {
            GamePhase::Attack => {
                debug("AI attempting to attack");
                let attack_result = handle_ai_attack(app, current_player_idx);
                debug(format!("AI attack result: {:?}", attack_result));
                if *app.game_state.game_phase() == GamePhase::Defense {
                    let defender_idx = app.game_state.current_defender();
                    let is_human_defender =
                        app.game_state.players()[defender_idx].player_type() == &PlayerType::Human;
                    if is_human_defender {
                        debug("Human needs to defend, ending AI processing");
                        return;
                    } else {
                        debug("AI needs to defend against itself, continuing");
                        continue; // Process the defense in the next iteration
                    }
                } else if *app.game_state.game_phase() == GamePhase::Attack {
                    debug("AI passed attack, transitioning to drawing phase");
                    app.game_state.draw_cards();
                }
            }
            GamePhase::Defense => {
                debug("AI attempting to defend");
                let defense_result = handle_ai_defense(app, current_player_idx);
                debug(format!("AI defense result: {:?}", defense_result));
                if *app.game_state.game_phase() == GamePhase::Defense {
                    let current_defender = app.game_state.current_defender();
                    let is_ai_defender = app.game_state.players()[current_defender].player_type()
                        == &PlayerType::Computer;
                    if current_defender != current_player_idx {
                        debug("Pass occurred, roles have changed");
                        if is_ai_defender {
                            debug("AI passed to AI, continuing defense");
                            continue;
                        } else {
                            debug("AI passed to human, ending AI processing");
                            return;
                        }
                    } else {
                        debug("AI defense incomplete, forcing draw phase");
                        app.game_state.draw_cards();
                    }
                } else if *app.game_state.game_phase() == GamePhase::Drawing {
                    debug("AI defense complete, already in drawing phase");
                    app.game_state.draw_cards();
                }
            }
            GamePhase::Drawing => {
                debug("AI processing drawing phase");
                app.game_state.draw_cards();

                if *app.game_state.game_phase() == GamePhase::Attack {
                    let attacker_idx = app.game_state.current_attacker();
                    let is_human_attacker =
                        app.game_state.players()[attacker_idx].player_type() == &PlayerType::Human;
                    if is_human_attacker {
                        debug("Human's turn after drawing, ending AI processing");
                        return;
                    } else {
                        debug("AI's turn to attack after drawing, continuing to next iteration");
                        continue;
                    }
                }
            }
            _ => {
                debug("AI turn in unhandled game phase, ending AI processing");
                return;
            }
        }
        if *app.game_state.game_phase() == GamePhase::Drawing {
            debug("Handling drawing phase transition");
            app.game_state.draw_cards();
            if *app.game_state.game_phase() == GamePhase::Drawing {
                debug("Forcing transition from Drawing to Attack phase");
                app.game_state = crate::game::GameState::force_attack_phase(app.game_state.clone());
            }
            if *app.game_state.game_phase() == GamePhase::Attack {
                let attacker_idx = app.game_state.current_attacker();
                let is_human_attacker =
                    app.game_state.players()[attacker_idx].player_type() == &PlayerType::Human;

                if is_human_attacker {
                    debug("Human's turn after drawing phase completed, ending AI processing");
                    return;
                }
            }
        }
        if app.game_state.check_game_over() {
            app.app_state = super::state::AppState::GameOver;
            return;
        }
        if turn_counter >= MAX_TURNS - 1 {
            debug("Reached maximum AI turn iterations, forcing end to prevent issues");
            if *app.game_state.game_phase() == GamePhase::Drawing {
                app.game_state = crate::game::GameState::force_attack_phase(app.game_state.clone());
            }
            return;
        }
    }
}
/// Handle AI attack phase
fn handle_ai_attack(app: &mut App, player_idx: usize) -> Result<(), String> {
    debug("AI is attacking");
    if *app.game_state.game_phase() != GamePhase::Attack {
        debug("AI called to attack but not in attack phase");
        return Ok(());
    }
    if app.game_state.current_attacker() != player_idx {
        debug(format!(
            "Wrong player attacking: expected {}, got {}",
            app.game_state.current_attacker(),
            player_idx
        ));
        return Ok(());
    }
    // Get attack moves from AI
    let attack_cards = app.ai_player.make_attack_move(&app.game_state, player_idx);
    if let Some(cards) = attack_cards {
        if cards.is_empty() {
            debug("AI decided to pass");
            return Ok(()); // AI passes
        }
        // Sort and make attacks (highest index first to prevent shifting)
        let mut sorted_indices: Vec<usize> = cards.iter().map(|(idx, _)| *idx).collect();
        sorted_indices.sort_by(|a, b| b.cmp(a));
        let mut attack_successful = false;
        for &idx in sorted_indices.iter() {
            match app.game_state.attack(idx, player_idx) {
                Ok(_) => {
                    attack_successful = true;
                    debug(format!("AI successfully attacked with card {}", idx));
                }
                Err(e) => {
                    debug(format!("AI attack failed: {}", e));
                    return Err(e.to_string());
                }
            }
        }
        if attack_successful {
            debug(format!(
                "AI successfully attacked with {} cards",
                sorted_indices.len()
            ));
            // Verify we've transitioned to defense phase
            if *app.game_state.game_phase() != GamePhase::Defense {
                debug("Warning: Game did not transition to Defense phase after successful attack");
                let defender_idx = (player_idx + 1) % app.game_state.players().len();
                app.game_state
                    .set_phase_to_defense(player_idx, defender_idx);
            }
        }
    } else {
        debug("AI decided to pass (no attacks)");
    }
    Ok(())
}
/// Handle AI defense phase
fn handle_ai_defense(app: &mut App, player_idx: usize) -> Result<(), String> {
    debug("AI is defending");
    // Check if the game state is valid for defense
    if *app.game_state.game_phase() != GamePhase::Defense {
        debug("AI called to defend but not in defense phase");
        return Ok(());
    }
    // Verify the correct player is defending
    if app.game_state.current_defender() != player_idx {
        debug(format!(
            "Wrong player defending: expected {}, got {}",
            app.game_state.current_defender(),
            player_idx
        ));
        return Ok(());
    }
    // Check if AI should take cards instead of defending
    if app.ai_player.should_take_cards(&app.game_state, player_idx) {
        debug("AI decided to take cards");
        if let Err(e) = app.game_state.take_cards() {
            return Err(e.to_string());
        }
        return Ok(());
    }
    // Get the table state
    let table_cards = app.game_state.table_cards();
    if table_cards.is_empty() {
        debug("No cards to defend against");
        return Ok(());
    }
    // Check for undefended attacks
    let has_undefended = table_cards.iter().any(|(_, defense)| defense.is_none());
    if !has_undefended {
        debug("All attacks already defended");
        return Ok(());
    }
    // Try to defend each undefended attack one at a time
    let mut defense_failed = false;
    while !defense_failed
        && app
            .game_state
            .table_cards()
            .iter()
            .any(|(_, d)| d.is_none())
    {
        if let Some(defense_cards) = app.ai_player.make_defense_move(&app.game_state, player_idx) {
            debug(format!("AI defending with cards: {:?}", defense_cards));
            // Process each defense card
            for (_table_idx, card) in &defense_cards {
                // We need to find the hand index of this card
                if let Some(hand_idx) = app.find_card_index_in_hand(player_idx, *card) {
                    match app.game_state.defend(hand_idx) {
                        Ok(_) => {
                            debug(format!("AI successfully defended with card {}", hand_idx));
                            // Check if a pass occurred by looking at the defender change
                            if app.game_state.current_defender() != player_idx {
                                debug("AI passed the card to a different player");
                                return Ok(());
                            }
                            let all_defended = !app
                                .game_state
                                .table_cards()
                                .iter()
                                .any(|(_, defense)| defense.is_none());
                            if all_defended {
                                debug("AI successfully defended all attacks");
                                // Get all cards from the table for discarding
                                let cards_to_discard: Vec<(usize, crate::game::card::Card)> = app
                                    .game_state
                                    .table_cards()
                                    .iter()
                                    .enumerate()
                                    .filter_map(|(idx, (_, defense))| {
                                        defense.map(|card| (idx, card))
                                    })
                                    .collect();
                                // Discard the cards
                                app.game_state.discard_cards(cards_to_discard);
                                return Ok(());
                            }
                        }
                        Err(e) => {
                            debug(format!("AI defense failed: {}", e));
                            defense_failed = true;
                            break;
                        }
                    }
                } else {
                    debug("Card not found in AI's hand");
                    defense_failed = true;
                    break;
                }
            }
        } else {
            debug("AI cannot defend further");
            defense_failed = true;
        }
    }
    if defense_failed
        || app
            .game_state
            .table_cards()
            .iter()
            .any(|(_, d)| d.is_none())
    {
        debug("AI taking cards after failed defense");
        if let Err(e) = app.game_state.take_cards() {
            return Err(e.to_string());
        }
    }
    Ok(())
}
