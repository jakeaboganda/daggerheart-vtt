//! WebSocket connection handling - Phase 5A: Refactored for Character/Connection architecture

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::sync::broadcast;
use uuid::Uuid;

use daggerheart_engine::character::{Ancestry, Attributes, Class};

use crate::{
    game::{self, GameState, SharedGameState},
    protocol::{self, CharacterInfo, ClientMessage, ServerMessage},
};

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

    // Create a new connection
    let conn_id = {
        let mut game = state.game.write().await;
        let conn = game.add_connection();
        conn.id
    };

    println!("📡 New connection: {}", conn_id);

    // Send connection established message
    let msg = ServerMessage::Connected {
        connection_id: conn_id.to_string(),
    };
    let _ = sender.send(Message::Text(msg.to_json())).await;

    // Send current characters list
    send_characters_list(&state, &conn_id, &mut sender).await;

    // Spawn task to forward broadcasts to this client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Main message processing loop
    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                handle_client_message(&state_clone, &conn_id, &text).await;
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    // Clean up connection on disconnect
    println!("👋 Connection disconnected: {}", conn_id);
    let mut game = state.game.write().await;

    // Get controlled character before removing connection
    let controlled_char_id = game.control_mapping.get(&conn_id).copied();

    game.remove_connection(&conn_id);

    // Broadcast updated characters list
    drop(game);
    broadcast_characters_list(&state).await;

    println!(
        "   Connection {} removed, controlled character: {:?}",
        conn_id, controlled_char_id
    );
}

/// Handle a client message
async fn handle_client_message(state: &AppState, conn_id: &Uuid, text: &str) {
    let msg: ClientMessage = match serde_json::from_str(text) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("❌ Failed to parse message: {}", e);
            return;
        }
    };

    match msg {
        ClientMessage::Connect => {
            // Already handled in handle_socket
        }

        ClientMessage::CreateCharacter {
            name,
            class,
            ancestry,
            attributes,
        } => {
            handle_create_character(state, conn_id, name, class, ancestry, attributes).await;
        }

        ClientMessage::SelectCharacter { character_id } => {
            handle_select_character(state, conn_id, character_id).await;
        }

        ClientMessage::MoveCharacter { x, y } => {
            handle_move_character(state, conn_id, x, y).await;
        }

        ClientMessage::RollDuality {
            modifier,
            with_advantage,
        } => {
            handle_roll_duality(state, conn_id, modifier, with_advantage).await;
        }

        ClientMessage::UpdateResource { resource, amount } => {
            handle_update_resource(state, conn_id, resource, amount).await;
        }

        ClientMessage::RequestRoll {
            target_type,
            target_character_ids,
            roll_type,
            attribute,
            difficulty,
            context,
            narrative_stakes,
            situational_modifier,
            has_advantage,
            is_combat,
        } => {
            handle_request_roll(
                state,
                target_type,
                target_character_ids,
                roll_type,
                attribute,
                difficulty,
                context,
                narrative_stakes,
                situational_modifier,
                has_advantage,
                is_combat,
            )
            .await;
        }

        ClientMessage::ExecuteRoll {
            request_id,
            spend_hope_for_bonus,
            chosen_experience,
        } => {
            handle_execute_roll(
                state,
                conn_id,
                request_id,
                spend_hope_for_bonus,
                chosen_experience,
            )
            .await;
        }

        // ===== Combat & Adversary Handlers =====
        
        ClientMessage::SpawnAdversary { template, position } => {
            handle_spawn_adversary(state, template, position).await;
        }

        ClientMessage::SpawnCustomAdversary {
            name,
            position,
            hp,
            evasion,
            armor,
            attack_modifier,
            damage_dice,
        } => {
            handle_spawn_custom_adversary(
                state,
                name,
                position,
                hp,
                evasion,
                armor,
                attack_modifier,
                damage_dice,
            )
            .await;
        }

        ClientMessage::RemoveAdversary { adversary_id } => {
            handle_remove_adversary(state, adversary_id).await;
        }

        ClientMessage::StartCombat => {
            handle_start_combat(state).await;
        }

        ClientMessage::EndCombat => {
            handle_end_combat(state).await;
        }

        ClientMessage::AddTrackerToken { token_type } => {
            handle_add_tracker_token(state, token_type).await;
        }

        ClientMessage::Attack {
            attacker_id,
            target_id,
            modifier,
            with_advantage,
        } => {
            handle_attack(state, attacker_id, target_id, modifier, with_advantage).await;
        }

        ClientMessage::RollDamage {
            attacker_id,
            target_id,
            damage_dice,
            armor,
        } => {
            handle_roll_damage(state, attacker_id, target_id, damage_dice, armor).await;
        }
    }
}

/// Handle character creation
async fn handle_create_character(
    state: &AppState,
    conn_id: &Uuid,
    name: String,
    class_str: String,
    ancestry_str: String,
    attributes: [i8; 6],
) {
    let class = match class_str.as_str() {
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
            send_error(state, &format!("Invalid class: {}", class_str)).await;
            return;
        }
    };

    let ancestry = match ancestry_str.as_str() {
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
            send_error(state, &format!("Invalid ancestry: {}", ancestry_str)).await;
            return;
        }
    };

    let attrs = match Attributes::from_array(attributes) {
        Ok(a) => a,
        Err(e) => {
            send_error(state, &format!("Invalid attributes: {}", e)).await;
            return;
        }
    };

    let mut game = state.game.write().await;
    let character = game.create_character(name, class, ancestry, attrs);
    let char_id = character.id;

    println!("✨ Character created: {} ({})", character.name, char_id);
    
    // Log event
    game.add_event(
        game::GameEventType::CharacterCreated,
        format!("{} joined the game", character.name),
        Some(character.name.clone()),
        Some(format!("Class: {}, Ancestry: {}", class_str, ancestry_str)),
    );
    
    let event = game.event_log.last().cloned();

    // Auto-select the newly created character
    if let Err(e) = game.select_character(conn_id, &char_id) {
        eprintln!("❌ Failed to auto-select character: {}", e);
        drop(game);
        send_error(state, &format!("Failed to select character: {}", e)).await;
        return;
    }

    let character_data = character.to_data();
    drop(game);
    
    // Broadcast event
    if let Some(ev) = event {
        broadcast_event(state, &ev).await;
    }

    // Broadcast character spawned
    let spawn_msg = ServerMessage::CharacterSpawned {
        character_id: char_id.to_string(),
        name: character_data.name.clone(),
        position: character.position,
        color: character.color.clone(),
        is_npc: false,
    };
    let _ = state.broadcaster.send(spawn_msg.to_json());

    // Send character created confirmation to creator
    let created_msg = ServerMessage::CharacterCreated {
        character_id: char_id.to_string(),
        character: character_data.clone(),
    };
    let _ = state.broadcaster.send(created_msg.to_json());

    // Send character selected message
    let selected_msg = ServerMessage::CharacterSelected {
        character_id: char_id.to_string(),
        character: character_data,
    };
    let _ = state.broadcaster.send(selected_msg.to_json());

    // Broadcast updated characters list
    broadcast_characters_list(state).await;
}

/// Handle character selection
async fn handle_select_character(state: &AppState, conn_id: &Uuid, character_id: String) {
    let char_uuid = match Uuid::parse_str(&character_id) {
        Ok(id) => id,
        Err(_) => {
            send_error(state, "Invalid character ID").await;
            return;
        }
    };

    let mut game = state.game.write().await;

    if let Err(e) = game.select_character(conn_id, &char_uuid) {
        drop(game);
        send_error(state, &format!("Failed to select character: {}", e)).await;
        return;
    }

    let character = match game.get_character(&char_uuid) {
        Some(c) => c.clone(),
        None => {
            drop(game);
            send_error(state, "Character not found").await;
            return;
        }
    };

    let character_data = character.to_data();
    drop(game);

    println!(
        "🎮 Connection {} selected character: {}",
        conn_id, character.name
    );

    // Send character selected confirmation
    let msg = ServerMessage::CharacterSelected {
        character_id: char_uuid.to_string(),
        character: character_data,
    };
    let _ = state.broadcaster.send(msg.to_json());

    // Broadcast updated characters list
    broadcast_characters_list(state).await;
}

/// Handle character movement
async fn handle_move_character(state: &AppState, conn_id: &Uuid, x: f32, y: f32) {
    let game = state.game.read().await;

    let char_id = match game.control_mapping.get(conn_id) {
        Some(id) => *id,
        None => {
            drop(game);
            send_error(state, "No character selected").await;
            return;
        }
    };
    drop(game);

    let mut game = state.game.write().await;
    let position = crate::protocol::Position::new(x, y);

    if !game.update_character_position(&char_id, position) {
        drop(game);
        send_error(state, "Failed to update position").await;
        return;
    }
    drop(game);

    // Broadcast movement
    let msg = ServerMessage::CharacterMoved {
        character_id: char_id.to_string(),
        position,
    };
    let _ = state.broadcaster.send(msg.to_json());
}

/// Handle dice roll
async fn handle_roll_duality(
    state: &AppState,
    conn_id: &Uuid,
    modifier: i32,
    with_advantage: bool,
) {
    let game = state.game.read().await;

    let char_id = match game.control_mapping.get(conn_id) {
        Some(id) => *id,
        None => {
            drop(game);
            send_error(state, "No character selected").await;
            return;
        }
    };

    let character = match game.get_character(&char_id) {
        Some(c) => c.clone(),
        None => {
            drop(game);
            send_error(state, "Character not found").await;
            return;
        }
    };

    let roll = game.roll_duality(modifier, with_advantage);
    drop(game);

    println!(
        "🎲 {} rolled: {}d12 = {}",
        character.name, roll.hope, roll.fear
    );

    // Broadcast roll result
    let msg = ServerMessage::RollResult {
        character_id: char_id.to_string(),
        character_name: character.name,
        roll,
    };
    let _ = state.broadcaster.send(msg.to_json());
}

/// Handle resource update
async fn handle_update_resource(state: &AppState, conn_id: &Uuid, resource: String, amount: i32) {
    let game = state.game.read().await;

    let char_id = match game.control_mapping.get(conn_id) {
        Some(id) => *id,
        None => {
            drop(game);
            send_error(state, "No character selected").await;
            return;
        }
    };
    drop(game);

    let mut game = state.game.write().await;

    let character = match game.get_character_mut(&char_id) {
        Some(c) => c,
        None => {
            drop(game);
            send_error(state, "Character not found").await;
            return;
        }
    };

    match resource.as_str() {
        "hp" => {
            if amount < 0 {
                character.hp.take_damage((-amount) as u8);
            } else {
                character.hp.heal(amount as u8);
            }
        }
        "stress" => {
            if amount > 0 {
                character.stress.gain(amount as u8);
            } else {
                character.stress.clear();
            }
        }
        "hope" => {
            if amount < 0 {
                let _ = character.hope.spend((-amount) as u8);
            } else {
                character.hope.gain(amount as u8);
            }
        }
        _ => {
            drop(game);
            send_error(state, &format!("Invalid resource: {}", resource)).await;
            return;
        }
    }

    character.sync_resources();
    let character_data = character.to_data();
    drop(game);

    // Broadcast character update
    let msg = ServerMessage::CharacterUpdated {
        character_id: char_id.to_string(),
        character: character_data,
    };
    let _ = state.broadcaster.send(msg.to_json());
}

/// Send error message
async fn send_error(state: &AppState, message: &str) {
    let msg = ServerMessage::Error {
        message: message.to_string(),
    };
    let _ = state.broadcaster.send(msg.to_json());
}

/// Broadcast a game event to all clients
async fn broadcast_event(state: &AppState, event: &game::GameEvent) {
    use std::time::UNIX_EPOCH;
    
    let timestamp = event.timestamp
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    let timestamp_str = chrono::DateTime::from_timestamp(timestamp as i64, 0)
        .map(|dt| dt.format("%H:%M:%S").to_string())
        .unwrap_or_else(|| "??:??:??".to_string());
    
    let event_type_str = format!("{:?}", event.event_type);
    
    let msg = protocol::ServerMessage::GameEvent {
        timestamp: timestamp_str,
        event_type: event_type_str,
        message: event.message.clone(),
        character_name: event.character_name.clone(),
        details: event.details.clone(),
    };
    
    let _ = state.broadcaster.send(msg.to_json());
}

/// Send characters list to a specific connection
async fn send_characters_list(
    state: &AppState,
    conn_id: &Uuid,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) {
    let game = state.game.read().await;
    let characters = build_character_list(&game, conn_id);
    drop(game);

    let msg = ServerMessage::CharactersList { characters };
    let _ = sender.send(Message::Text(msg.to_json())).await;
}

/// Broadcast characters list to all connections
async fn broadcast_characters_list(_state: &AppState) {
    // We can't personalize broadcasts, so we'll send a generic list
    // Clients will need to request full details separately if needed
    // For now, just notify that the list changed
    // TODO: This could be optimized by sending the full list to each connection individually
}

/// Build character list with control information for a specific connection
fn build_character_list(game: &GameState, conn_id: &Uuid) -> Vec<CharacterInfo> {
    let my_char_id = game.control_mapping.get(conn_id).copied();

    game.get_characters()
        .iter()
        .map(|character| {
            let controlled_by_me = Some(character.id) == my_char_id;
            let controlled_by_other = game
                .control_mapping
                .values()
                .any(|&char_id| char_id == character.id && Some(char_id) != my_char_id);

            CharacterInfo {
                id: character.id.to_string(),
                name: character.name.clone(),
                class: character.class.to_string(),
                ancestry: character.ancestry.to_string(),
                position: character.position,
                color: character.color.clone(),
                is_npc: character.is_npc,
                controlled_by_me,
                controlled_by_other,
            }
        })
        .collect()
}

// ===== Phase 1: GM-Initiated Dice Rolls =====

/// Handle GM roll request
async fn handle_request_roll(
    state: &AppState,
    target_type: protocol::RollTargetType,
    target_character_ids: Vec<String>,
    roll_type: protocol::RollType,
    attribute: Option<String>,
    difficulty: u16,
    context: String,
    narrative_stakes: Option<String>,
    situational_modifier: i8,
    has_advantage: bool,
    is_combat: bool,
) {
    use uuid::Uuid;

    let mut game = state.game.write().await;

    // Parse target character IDs
    let mut target_uuids = Vec::new();
    match target_type {
        protocol::RollTargetType::Specific => {
            for id_str in &target_character_ids {
                if let Ok(uuid) = Uuid::parse_str(id_str) {
                    if game.characters.contains_key(&uuid) {
                        target_uuids.push(uuid);
                    }
                }
            }
        }
        protocol::RollTargetType::All => {
            target_uuids = game.get_player_characters().iter().map(|c| c.id).collect();
        }
        protocol::RollTargetType::Npc => {
            // For MVP, treat as specific
            for id_str in &target_character_ids {
                if let Ok(uuid) = Uuid::parse_str(id_str) {
                    if game.characters.contains_key(&uuid) {
                        target_uuids.push(uuid);
                    }
                }
            }
        }
    }

    if target_uuids.is_empty() {
        send_error(state, "No valid characters targeted").await;
        return;
    }

    // Create roll request
    let request_id = Uuid::new_v4().to_string();
    let request = game::PendingRollRequest {
        id: request_id.clone(),
        target_character_ids: target_uuids.clone(),
        roll_type: roll_type.clone(),
        attribute: attribute.clone(),
        difficulty,
        context: context.clone(),
        narrative_stakes: narrative_stakes.clone(),
        situational_modifier,
        has_advantage,
        is_combat,
        completed_by: Vec::new(),
        timestamp: std::time::SystemTime::now(),
    };

    game.pending_roll_requests
        .insert(request_id.clone(), request);
    
    // Log event
    let target_names: Vec<String> = target_uuids
        .iter()
        .filter_map(|id| game.characters.get(id).map(|c| c.name.clone()))
        .collect();
    let target_desc = if target_names.len() == game.get_player_characters().len() {
        "all players".to_string()
    } else {
        target_names.join(", ")
    };
    
    game.add_event(
        game::GameEventType::RollRequested,
        format!("GM requested {} roll: \"{}\"", 
            attribute.as_deref().unwrap_or("general"),
            context
        ),
        None,
        Some(format!("Target: {}, DC {}", target_desc, difficulty)),
    );

    // Send roll request to each targeted character
    for char_id in &target_uuids {
        if let Some(character) = game.characters.get(char_id) {
            // Calculate base modifier
            let attr_mod = if let Some(ref attr) = attribute {
                character.get_attribute(attr).unwrap_or(0)
            } else {
                0
            };

            let prof_mod = match roll_type {
                protocol::RollType::Attack | protocol::RollType::Spellcast => {
                    character.proficiency_bonus()
                }
                _ => 0,
            };

            let base_modifier = attr_mod + prof_mod;
            let total_modifier = base_modifier + situational_modifier;

            let can_spend_hope = character.hope.current >= 1 && !character.experiences.is_empty();

            let msg = protocol::ServerMessage::RollRequested {
                request_id: request_id.clone(),
                roll_type: roll_type.clone(),
                attribute: attribute.clone(),
                difficulty,
                context: context.clone(),
                narrative_stakes: narrative_stakes.clone(),
                base_modifier,
                situational_modifier,
                total_modifier,
                has_advantage,
                your_attribute_value: attr_mod,
                your_proficiency: prof_mod,
                can_spend_hope,
                experiences: character.experiences.clone(),
            };

            state.broadcaster.send(msg.to_json()).ok();
        }
    }

    // Send status to GM
    let pending: Vec<String> = target_uuids
        .iter()
        .filter_map(|id| game.characters.get(id).map(|c| c.name.clone()))
        .collect();

    let status_msg = protocol::ServerMessage::RollRequestStatus {
        request_id,
        pending_characters: pending,
        completed_characters: Vec::new(),
    };

    state.broadcaster.send(status_msg.to_json()).ok();
}

/// Handle player executing a roll
async fn handle_execute_roll(
    state: &AppState,
    conn_id: &Uuid,
    request_id: String,
    spend_hope: bool,
    chosen_experience: Option<String>,
) {
    let mut game = state.game.write().await;

    // Get character ID for this connection
    let char_id = match game.control_mapping.get(conn_id) {
        Some(id) => *id,
        None => {
            send_error(state, "No character controlled").await;
            return;
        }
    };

    // Execute the roll
    let roll_result = match game.execute_roll(&char_id, &request_id, spend_hope) {
        Ok(result) => result,
        Err(e) => {
            send_error(state, &e).await;
            return;
        }
    };

    // Get character name and request context
    let character_name = game
        .characters
        .get(&char_id)
        .map(|c| c.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let request = game.pending_roll_requests.get(&request_id).cloned();
    let context = request
        .as_ref()
        .map(|r| r.context.clone())
        .unwrap_or_default();
    let roll_type = request
        .as_ref()
        .map(|r| r.roll_type.clone())
        .unwrap_or(protocol::RollType::Action);

    // Get new Hope/Fear values
    let character = game.characters.get(&char_id).unwrap();
    let new_hope = character.hope.current;
    let new_fear = game.fear_pool;

    // Create outcome description
    let outcome_description = match roll_result.success_type {
        protocol::SuccessType::CriticalSuccess => "CRITICAL SUCCESS".to_string(),
        protocol::SuccessType::SuccessWithHope => "SUCCESS WITH HOPE".to_string(),
        protocol::SuccessType::SuccessWithFear => "SUCCESS WITH FEAR".to_string(),
        protocol::SuccessType::Failure => "FAILURE".to_string(),
    };
    
    // Log event
    let roll_message = format!(
        "{} rolled {} for \"{}\"",
        character_name,
        outcome_description.to_lowercase(),
        context
    );
    let roll_details = format!(
        "Hope: {}, Fear: {}, Total: {}",
        roll_result.hope_die,
        roll_result.fear_die,
        roll_result.total
    );
    game.add_event(
        game::GameEventType::RollExecuted,
        roll_message,
        Some(character_name.clone()),
        Some(roll_details),
    );
    let event = game.event_log.last().cloned();

    // Broadcast result to all clients
    let msg = protocol::ServerMessage::DetailedRollResult {
        request_id: request_id.clone(),
        character_id: char_id.to_string(),
        character_name,
        roll_type,
        context,
        roll_details: roll_result,
        outcome_description,
        new_hope,
        new_fear,
    };

    state.broadcaster.send(msg.to_json()).ok();

    // Update roll request status
    if let Some(req) = game.pending_roll_requests.get(&request_id) {
        let pending: Vec<String> = req
            .target_character_ids
            .iter()
            .filter(|id| !req.completed_by.contains(id))
            .filter_map(|id| game.characters.get(id).map(|c| c.name.clone()))
            .collect();

        let completed: Vec<String> = req
            .completed_by
            .iter()
            .filter_map(|id| game.characters.get(id).map(|c| c.name.clone()))
            .collect();

        let status_msg = protocol::ServerMessage::RollRequestStatus {
            request_id,
            pending_characters: pending,
            completed_characters: completed,
        };

        state.broadcaster.send(status_msg.to_json()).ok();
    }

    // Broadcast updated character data
    if let Some(character) = game.characters.get(&char_id).cloned() {
        let msg = protocol::ServerMessage::CharacterUpdated {
            character_id: char_id.to_string(),
            character: character.to_data(),
        };
        state.broadcaster.send(msg.to_json()).ok();
    }
    
    drop(game);
    
    // Broadcast event
    if let Some(ev) = event {
        broadcast_event(state, &ev).await;
    }
}

// ===== Combat & Adversary Handlers =====

/// Handle spawning an adversary from template
async fn handle_spawn_adversary(state: &AppState, template: String, position: protocol::Position) {
    let mut game = state.game.write().await;
    
    match game.spawn_adversary(&template, position) {
        Ok(adversary) => {
            // Broadcast adversary spawned
            let msg = ServerMessage::AdversarySpawned {
                adversary_id: adversary.id.clone(),
                name: adversary.name.clone(),
                template: adversary.template.clone(),
                position,
                hp: adversary.hp,
                max_hp: adversary.max_hp,
                evasion: adversary.evasion,
                armor: adversary.armor,
                attack_modifier: adversary.attack_modifier,
                damage_dice: adversary.damage_dice.clone(),
            };
            let _ = state.broadcaster.send(msg.to_json());
            
            // Broadcast event
            if let Some(event) = game.event_log.last() {
                broadcast_event(state, event).await;
            }
        }
        Err(e) => {
            send_error(state, &e).await;
        }
    }
}

/// Handle spawning a custom adversary
async fn handle_spawn_custom_adversary(
    state: &AppState,
    name: String,
    position: protocol::Position,
    hp: u8,
    evasion: u8,
    armor: u8,
    attack_modifier: i8,
    damage_dice: String,
) {
    let mut game = state.game.write().await;
    
    let adversary = game.create_custom_adversary(
        name,
        position,
        hp,
        evasion,
        armor,
        attack_modifier,
        damage_dice.clone(),
    );
    
    // Broadcast adversary spawned
    let msg = ServerMessage::AdversarySpawned {
        adversary_id: adversary.id.clone(),
        name: adversary.name.clone(),
        template: adversary.template.clone(),
        position,
        hp: adversary.hp,
        max_hp: adversary.max_hp,
        evasion: adversary.evasion,
        armor: adversary.armor,
        attack_modifier: adversary.attack_modifier,
        damage_dice: adversary.damage_dice.clone(),
    };
    let _ = state.broadcaster.send(msg.to_json());
    
    // Broadcast event
    if let Some(event) = game.event_log.last() {
        broadcast_event(state, event).await;
    }
}

/// Handle removing an adversary
async fn handle_remove_adversary(state: &AppState, adversary_id: String) {
    let mut game = state.game.write().await;
    
    if let Some(adversary) = game.remove_adversary(&adversary_id) {
        let msg = ServerMessage::AdversaryRemoved {
            adversary_id,
            name: adversary.name.clone(),
        };
        let _ = state.broadcaster.send(msg.to_json());
        
        // Broadcast event
        if let Some(event) = game.event_log.last() {
            broadcast_event(state, event).await;
        }
    }
}

/// Handle starting combat
async fn handle_start_combat(state: &AppState) {
    let mut game = state.game.write().await;
    
    let encounter_id = game.start_combat();
    
    if let Some(encounter) = game.get_combat() {
        let msg = ServerMessage::CombatStarted {
            encounter_id,
            pc_tokens: encounter.action_tracker.pc_tokens,
            adversary_tokens: encounter.action_tracker.adversary_tokens,
        };
        let _ = state.broadcaster.send(msg.to_json());
        
        // Broadcast event
        if let Some(event) = game.event_log.last() {
            broadcast_event(state, event).await;
        }
    }
}

/// Handle ending combat
async fn handle_end_combat(state: &AppState) {
    let mut game = state.game.write().await;
    
    game.end_combat("manual");
    
    let msg = ServerMessage::CombatEnded {
        reason: "manual".to_string(),
    };
    let _ = state.broadcaster.send(msg.to_json());
    
    // Broadcast event
    if let Some(event) = game.event_log.last() {
        broadcast_event(state, event).await;
    }
}

/// Handle adding a tracker token
async fn handle_add_tracker_token(state: &AppState, token_type: String) {
    let mut game = state.game.write().await;
    
    if let Some(encounter) = game.get_combat_mut() {
        match token_type.as_str() {
            "pc" => encounter.action_tracker.add_pc_token(),
            "adversary" => encounter.action_tracker.add_adversary_token(),
            _ => {
                send_error(state, &format!("Invalid token type: {}", token_type)).await;
                return;
            }
        }
        
        let next_token = encounter.action_tracker.get_next()
            .map(|t| format!("{:?}", t).to_lowercase())
            .unwrap_or_else(|| "none".to_string());
        
        let msg = ServerMessage::TrackerUpdated {
            pc_tokens: encounter.action_tracker.pc_tokens,
            adversary_tokens: encounter.action_tracker.adversary_tokens,
            next_token,
        };
        let _ = state.broadcaster.send(msg.to_json());
    }
}

/// Handle attack roll
async fn handle_attack(
    state: &AppState,
    attacker_id: String,
    target_id: String,
    modifier: i8,
    with_advantage: bool,
) {
    use daggerheart_engine::core::dice::duality::DualityRoll;
    
    let game = state.game.read().await;
    
    // Get attacker and target names
    let attacker_name = game.characters.values()
        .find(|c| c.id.to_string() == attacker_id)
        .map(|c| c.name.clone())
        .or_else(|| {
            game.adversaries.values()
                .find(|a| a.id == attacker_id)
                .map(|a| a.name.clone())
        })
        .unwrap_or_else(|| "Unknown".to_string());
    
    let target_name = game.characters.values()
        .find(|c| c.id.to_string() == target_id)
        .map(|c| c.name.clone())
        .or_else(|| {
            game.adversaries.values()
                .find(|a| a.id == target_id)
                .map(|a| a.name.clone())
        })
        .unwrap_or_else(|| "Unknown".to_string());
    
    let target_evasion = game.characters.values()
        .find(|c| c.id.to_string() == target_id)
        .map(|c| c.evasion as u8)
        .or_else(|| {
            game.adversaries.values()
                .find(|a| a.id == target_id)
                .map(|a| a.evasion)
        })
        .unwrap_or(10);
    
    // Roll attack
    let roll = DualityRoll::roll();
    let result = if with_advantage {
        roll.with_advantage()
    } else {
        roll.with_modifier(modifier)
    };
    
    let hope = result.roll.hope as u16;
    let fear = result.roll.fear as u16;
    let controlling_die = if hope > fear { "hope" } else { "fear" };
    let total = result.total as u16;
    let hit = total >= target_evasion as u16;
    let is_critical = result.is_critical;
    
    // Broadcast attack result
    let msg = ServerMessage::AttackResult {
        attacker_id: attacker_id.clone(),
        attacker_name: attacker_name.clone(),
        target_id: target_id.clone(),
        target_name: target_name.clone(),
        hope,
        fear,
        modifier,
        total,
        target_evasion,
        hit,
        controlling_die: controlling_die.to_string(),
        is_critical,
    };
    let _ = state.broadcaster.send(msg.to_json());
}

/// Handle damage roll
async fn handle_roll_damage(
    state: &AppState,
    _attacker_id: String,
    target_id: String,
    damage_dice: String,
    armor: u8,
) {
    use daggerheart_engine::combat::damage::DamageResult;
    
    // Parse and roll damage dice
    let raw_damage = parse_and_roll_dice(&damage_dice);
    
    // Calculate damage with threshold system
    let damage_result = DamageResult::calculate(raw_damage, armor);
    
    let mut game = state.game.write().await;
    
    // Get target name
    let target_name = game.characters.values()
        .find(|c| c.id.to_string() == target_id)
        .map(|c| c.name.clone())
        .or_else(|| {
            game.adversaries.values()
                .find(|a| a.id == target_id)
                .map(|a| a.name.clone())
        })
        .unwrap_or_else(|| "Unknown".to_string());
    
    // Apply damage to target
    let mut taken_out = false;
    let mut new_hp = 0;
    let mut new_stress = 0;
    
    if let Some(character) = game.characters.values_mut().find(|c| c.id.to_string() == target_id) {
        // Apply to character
        if damage_result.hp_lost > 0 {
            character.hp_current = character.hp_current.saturating_sub(damage_result.hp_lost);
        }
        if damage_result.stress_gained > 0 {
            character.stress_current = (character.stress_current + damage_result.stress_gained).min(character.hp_max);
        }
        new_hp = character.hp_current;
        new_stress = character.stress_current;
        
        if character.hp_current == 0 && character.stress_current >= character.hp_max {
            taken_out = true;
        }
    } else if let Some(adversary) = game.adversaries.values_mut().find(|a| a.id == target_id) {
        // Apply to adversary
        taken_out = adversary.take_damage(damage_result.hp_lost, damage_result.stress_gained);
        new_hp = adversary.hp;
        new_stress = adversary.stress;
    }
    
    // Broadcast damage result
    let msg = ServerMessage::DamageResult {
        target_id: target_id.clone(),
        target_name: target_name.clone(),
        raw_damage: damage_result.raw_damage,
        after_armor: damage_result.after_armor,
        hp_lost: damage_result.hp_lost,
        stress_gained: damage_result.stress_gained,
        new_hp,
        new_stress,
        taken_out,
    };
    let _ = state.broadcaster.send(msg.to_json());
    
    // Log event
    game.add_event(
        game::GameEventType::CombatAction,
        format!(
            "{} took {} damage ({} HP, {} Stress)",
            target_name, damage_result.after_armor, damage_result.hp_lost, damage_result.stress_gained
        ),
        Some(target_name),
        if taken_out {
            Some("Taken out!".to_string())
        } else {
            None
        },
    );
    
    if let Some(event) = game.event_log.last() {
        broadcast_event(state, event).await;
    }
}

/// Parse and roll damage dice (e.g., "1d8+2" or "2d6")
fn parse_and_roll_dice(dice_str: &str) -> u16 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    // Split on '+' or '-'
    let (dice_part, modifier) = if let Some(pos) = dice_str.find('+') {
        let (d, m) = dice_str.split_at(pos);
        (d, m[1..].parse::<i16>().unwrap_or(0))
    } else if let Some(pos) = dice_str.find('-') {
        let (d, m) = dice_str.split_at(pos);
        (d, -m[1..].parse::<i16>().unwrap_or(0))
    } else {
        (dice_str, 0)
    };
    
    // Parse "XdY" format
    if let Some(d_pos) = dice_part.find('d') {
        let (num_str, die_str) = dice_part.split_at(d_pos);
        let num_dice = num_str.parse::<u16>().unwrap_or(1);
        let die_size = die_str[1..].parse::<u16>().unwrap_or(6);
        
        let mut total = 0;
        for _ in 0..num_dice {
            total += rng.gen_range(1..=die_size);
        }
        
        (total as i16 + modifier).max(0) as u16
    } else {
        // Just a flat number
        dice_part.parse::<u16>().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[test]
    fn test_app_state_clone() {
        let game_state = Arc::new(RwLock::new(GameState::new()));
        let (broadcaster, _) = broadcast::channel::<String>(100);

        let state = AppState {
            game: game_state,
            broadcaster,
        };

        let cloned = state.clone();
        assert!(Arc::ptr_eq(&state.game, &cloned.game));
    }
}
