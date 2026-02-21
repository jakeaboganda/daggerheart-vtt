//! WebSocket connection handler

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use tokio::sync::broadcast;
use uuid::Uuid;

use daggerheart_engine::character::{Ancestry, Attributes, Class};

use crate::game::SharedGameState;
use crate::protocol::{ClientMessage, PlayerInfo, ServerMessage};

/// Broadcast channel for server messages
pub type Broadcaster = broadcast::Sender<String>;

/// Application state passed to handlers
#[derive(Clone)]
pub struct AppState {
    pub game: SharedGameState,
    pub broadcaster: Broadcaster,
}

/// Handle WebSocket upgrade request
pub async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle an individual WebSocket connection
async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to broadcasts
    let mut rx = state.broadcaster.subscribe();

    // Player ID for this connection
    let mut player_id: Option<Uuid> = None;

    // Spawn task to forward broadcasts to this client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Main receive loop
    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                // Parse client message
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(client_msg) => {
                        match client_msg {
                            ClientMessage::PlayerJoin { name } => {
                                // Add player to game state
                                let player = {
                                    let mut game = state_clone.game.write().await;
                                    game.add_player(name.clone())
                                };

                                player_id = Some(player.id);

                                tracing::info!(
                                    "Player joined: {} ({}) at {:?}",
                                    player.name,
                                    player.id,
                                    player.position
                                );

                                // Broadcast player joined with position and color
                                let msg = ServerMessage::PlayerJoined {
                                    player_id: player.id.to_string(),
                                    name: player.name.clone(),
                                    position: player.position,
                                    color: player.color.clone(),
                                };
                                let _ = state_clone.broadcaster.send(msg.to_json());

                                // Send current players list to all clients (including new one)
                                // This ensures new clients see existing players
                                let players = {
                                    let game = state_clone.game.read().await;
                                    game.get_players()
                                        .into_iter()
                                        .map(|p| PlayerInfo {
                                            player_id: p.id.to_string(),
                                            name: p.name.clone(),
                                            connected: p.connected,
                                            position: p.position,
                                            color: p.color.clone(),
                                            has_character: p.character.is_some(),
                                            character_name: p
                                                .character
                                                .as_ref()
                                                .map(|c| c.name.clone()),
                                        })
                                        .collect()
                                };

                                let list_msg = ServerMessage::PlayersList { players };
                                let _ = state_clone.broadcaster.send(list_msg.to_json());
                            }

                            ClientMessage::PlayerMove { x, y } => {
                                if let Some(pid) = player_id {
                                    // Update position in game state
                                    let position = crate::protocol::Position::new(x, y);
                                    let updated = {
                                        let mut game = state_clone.game.write().await;
                                        game.update_position(&pid, position)
                                    };

                                    if updated {
                                        tracing::debug!("Player {} moved to ({}, {})", pid, x, y);

                                        // Broadcast movement
                                        let msg = ServerMessage::PlayerMoved {
                                            player_id: pid.to_string(),
                                            position,
                                        };
                                        let _ = state_clone.broadcaster.send(msg.to_json());
                                    }
                                }
                            }

                            ClientMessage::CreateCharacter {
                                name,
                                class,
                                ancestry,
                                attributes,
                            } => {
                                if let Some(pid) = player_id {
                                    // Parse class and ancestry
                                    let class_enum = match class.as_str() {
                                        "Bard" => Class::Bard,
                                        "Druid" => Class::Druid,
                                        "Guardian" => Class::Guardian,
                                        "Ranger" => Class::Ranger,
                                        "Rogue" => Class::Rogue,
                                        "Seraph" => Class::Seraph,
                                        "Sorcerer" => Class::Sorcerer,
                                        "Warrior" => Class::Warrior,
                                        "Wizard" => Class::Wizard,
                                        _ => {
                                            tracing::warn!("Unknown class: {}", class);
                                            continue;
                                        }
                                    };

                                    let ancestry_enum = match ancestry.as_str() {
                                        "Clank" => Ancestry::Clank,
                                        "Daemon" => Ancestry::Daemon,
                                        "Drakona" => Ancestry::Drakona,
                                        "Dwarf" => Ancestry::Dwarf,
                                        "Faerie" => Ancestry::Faerie,
                                        "Faun" => Ancestry::Faun,
                                        "Fungril" => Ancestry::Fungril,
                                        "Galapa" => Ancestry::Galapa,
                                        "Giant" => Ancestry::Giant,
                                        "Goblin" => Ancestry::Goblin,
                                        "Halfling" => Ancestry::Halfling,
                                        "Human" => Ancestry::Human,
                                        "Inferis" => Ancestry::Inferis,
                                        "Katari" => Ancestry::Katari,
                                        "Orc" => Ancestry::Orc,
                                        "Ribbet" => Ancestry::Ribbet,
                                        "Simiah" => Ancestry::Simiah,
                                        _ => {
                                            tracing::warn!("Unknown ancestry: {}", ancestry);
                                            continue;
                                        }
                                    };

                                    // Validate and create attributes
                                    let attrs = match Attributes::from_array(attributes) {
                                        Ok(a) => a,
                                        Err(_) => {
                                            let error_msg = ServerMessage::Error {
                                                message: "Invalid attribute distribution. Must be exactly [+2, +1, +1, 0, 0, -1] in any order.".to_string(),
                                            };
                                            let _ =
                                                state_clone.broadcaster.send(error_msg.to_json());
                                            continue;
                                        }
                                    };

                                    // Create character
                                    let character = {
                                        let mut game = state_clone.game.write().await;
                                        match game.create_character(
                                            &pid,
                                            name.clone(),
                                            class_enum,
                                            ancestry_enum,
                                            attrs,
                                        ) {
                                            Ok(c) => c,
                                            Err(e) => {
                                                let error_msg = ServerMessage::Error {
                                                    message: format!(
                                                        "Failed to create character: {}",
                                                        e
                                                    ),
                                                };
                                                let _ = state_clone
                                                    .broadcaster
                                                    .send(error_msg.to_json());
                                                continue;
                                            }
                                        }
                                    };

                                    tracing::info!(
                                        "Character created for player {}: {} ({} {})",
                                        pid,
                                        character.name,
                                        character.ancestry,
                                        character.class
                                    );

                                    // Broadcast character created
                                    let msg = ServerMessage::CharacterCreated {
                                        player_id: pid.to_string(),
                                        character: character.to_data(),
                                    };
                                    let _ = state_clone.broadcaster.send(msg.to_json());

                                    // Broadcast name update (so tokens display character name)
                                    let name_msg = ServerMessage::PlayerNameUpdated {
                                        player_id: pid.to_string(),
                                        display_name: character.name.clone(),
                                    };
                                    let _ = state_clone.broadcaster.send(name_msg.to_json());
                                }
                            }

                            ClientMessage::RollDuality {
                                modifier,
                                with_advantage,
                            } => {
                                if let Some(pid) = player_id {
                                    // Get display name (character name if available, otherwise player name)
                                    let player_name = {
                                        let game = state_clone.game.read().await;
                                        game.get_display_name(&pid)
                                            .unwrap_or_else(|| "Unknown".to_string())
                                    };

                                    // Roll dice
                                    let roll = {
                                        let game = state_clone.game.read().await;
                                        game.roll_duality(modifier, with_advantage)
                                    };

                                    tracing::info!("Player {} ({}) rolled: {} + {} (Hope) vs {} (Fear) = {}, controlling: {}",
                                        player_name, pid, modifier, roll.hope, roll.fear, roll.total, roll.controlling_die);

                                    // Broadcast roll result
                                    let msg = ServerMessage::RollResult {
                                        player_id: pid.to_string(),
                                        player_name,
                                        roll,
                                    };
                                    let _ = state_clone.broadcaster.send(msg.to_json());
                                }
                            }

                            ClientMessage::UpdateResource { resource, amount } => {
                                if let Some(pid) = player_id {
                                    let mut game = state_clone.game.write().await;
                                    if let Some(character) = game.get_character_mut(&pid) {
                                        match resource.as_str() {
                                            "hp" => {
                                                if amount > 0 {
                                                    character.hp.heal(amount as u8);
                                                } else {
                                                    character.hp.take_damage((-amount) as u8);
                                                }
                                            }
                                            "stress" => {
                                                if amount > 0 {
                                                    character.stress.gain(amount as u8);
                                                } else if amount < 0 {
                                                    // Clear stress
                                                    character.stress.clear();
                                                }
                                            }
                                            "hope" => {
                                                if amount > 0 {
                                                    character.hope.gain(amount as u8);
                                                } else if amount < 0 {
                                                    let _ = character.hope.spend((-amount) as u8);
                                                }
                                            }
                                            _ => {
                                                tracing::warn!("Unknown resource: {}", resource);
                                            }
                                        }

                                        // Broadcast character updated
                                        let msg = ServerMessage::CharacterUpdated {
                                            player_id: pid.to_string(),
                                            character: character.to_data(),
                                        };
                                        let _ = state_clone.broadcaster.send(msg.to_json());
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Invalid message: {}", e);
                    }
                }
            }
        }

        player_id
    });

    // Wait for tasks to complete
    tokio::select! {
        pid = &mut recv_task => {
            send_task.abort();

            // Player disconnected - remove from game
            if let Ok(Some(pid)) = pid {
                let removed = {
                    let mut game = state.game.write().await;
                    game.remove_player(&pid)
                };

                if let Some(player) = removed {
                    tracing::info!("Player left: {} ({})", player.name, player.id);

                    let msg = ServerMessage::PlayerLeft {
                        player_id: player.id.to_string(),
                        name: player.name,
                    };
                    let _ = state.broadcaster.send(msg.to_json());
                }
            }
        }
        _ = &mut send_task => {
            recv_task.abort();
        }
    }
}
