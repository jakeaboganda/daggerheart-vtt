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
    game::{GameState, SharedGameState},
    protocol::{CharacterInfo, ClientMessage, ServerMessage},
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
    
    println!("   Connection {} removed, controlled character: {:?}", conn_id, controlled_char_id);
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

    // Auto-select the newly created character
    if let Err(e) = game.select_character(conn_id, &char_id) {
        eprintln!("❌ Failed to auto-select character: {}", e);
        drop(game);
        send_error(state, &format!("Failed to select character: {}", e)).await;
        return;
    }

    let character_data = character.to_data();
    drop(game);

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

    println!("🎮 Connection {} selected character: {}", conn_id, character.name);

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

    println!("🎲 {} rolled: {}d12 = {}", character.name, roll.hope, roll.fear);

    // Broadcast roll result
    let msg = ServerMessage::RollResult {
        character_id: char_id.to_string(),
        character_name: character.name,
        roll,
    };
    let _ = state.broadcaster.send(msg.to_json());
}

/// Handle resource update
async fn handle_update_resource(
    state: &AppState,
    conn_id: &Uuid,
    resource: String,
    amount: i32,
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
            let controlled_by_other = game.control_mapping
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
