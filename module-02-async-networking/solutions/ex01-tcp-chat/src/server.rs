use crate::message::Message;
use crate::room::Room;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

const LOBBY_ROOM: &str = "lobby";

/// Shared server state
pub struct ChatServer {
    rooms: Arc<RwLock<HashMap<String, Room>>>,
    users: Arc<RwLock<HashMap<String, UserInfo>>>,
    max_connections: usize,
}

/// Information about a connected user
#[derive(Clone)]
pub struct UserInfo {
    pub nickname: String,
    pub current_room: String,
    pub tx: mpsc::UnboundedSender<Message>,
}

impl ChatServer {
    pub fn new(max_connections: usize) -> Self {
        let mut rooms = HashMap::new();
        rooms.insert(LOBBY_ROOM.to_string(), Room::new(LOBBY_ROOM.to_string()));

        ChatServer {
            rooms: Arc::new(RwLock::new(rooms)),
            users: Arc::new(RwLock::new(HashMap::new())),
            max_connections,
        }
    }

    /// Check if nickname is already taken
    pub async fn is_nickname_taken(&self, nickname: &str) -> bool {
        let users = self.users.read().await;
        users.contains_key(nickname)
    }

    /// Register a new user
    pub async fn register_user(
        &self,
        nickname: String,
        tx: mpsc::UnboundedSender<Message>,
    ) -> Result<(), String> {
        let mut users = self.users.write().await;

        if users.len() >= self.max_connections {
            return Err("Server is full".to_string());
        }

        if users.contains_key(&nickname) {
            return Err("Nickname already taken".to_string());
        }

        let user_info = UserInfo {
            nickname: nickname.clone(),
            current_room: LOBBY_ROOM.to_string(),
            tx,
        };

        users.insert(nickname.clone(), user_info);

        // Add to lobby
        let mut rooms = self.rooms.write().await;
        if let Some(lobby) = rooms.get_mut(LOBBY_ROOM) {
            lobby.add_member(nickname);
        }

        Ok(())
    }

    /// Unregister a user (on disconnect)
    pub async fn unregister_user(&self, nickname: &str) {
        let mut users = self.users.write().await;

        if let Some(user_info) = users.remove(nickname) {
            // Remove from their current room
            let mut rooms = self.rooms.write().await;
            if let Some(room) = rooms.get_mut(&user_info.current_room) {
                room.remove_member(nickname);

                // Notify room
                let msg = Message::system(format!("{} left the room", nickname));
                room.broadcast(msg);

                // Clean up empty rooms (except lobby)
                if room.is_empty() && user_info.current_room != LOBBY_ROOM {
                    rooms.remove(&user_info.current_room);
                }
            }
        }
    }

    /// Change user's nickname
    pub async fn change_nickname(&self, old_nick: &str, new_nick: String) -> Result<(), String> {
        let mut users = self.users.write().await;

        // Check if new nickname is taken
        if users.contains_key(&new_nick) {
            return Err("Nickname already taken".to_string());
        }

        // Get user info
        let mut user_info = users
            .remove(old_nick)
            .ok_or("User not found".to_string())?;

        let current_room = user_info.current_room.clone();

        // Update nickname
        user_info.nickname = new_nick.clone();

        // Reinsert with new nickname
        users.insert(new_nick.clone(), user_info);

        drop(users);

        // Update room membership
        let mut rooms = self.rooms.write().await;
        if let Some(room) = rooms.get_mut(&current_room) {
            room.remove_member(old_nick);
            room.add_member(new_nick.clone());

            // Notify room
            let msg = Message::system(format!("{} is now known as {}", old_nick, new_nick));
            room.broadcast(msg);
        }

        Ok(())
    }

    /// Move user to a different room
    pub async fn join_room(&self, nickname: &str, room_name: String) -> Result<(), String> {
        let mut users = self.users.write().await;
        let mut rooms = self.rooms.write().await;

        // Get user info
        let user_info = users
            .get_mut(nickname)
            .ok_or("User not found".to_string())?;

        let old_room_name = user_info.current_room.clone();

        // Can't join same room
        if old_room_name == room_name {
            return Err("Already in that room".to_string());
        }

        // Remove from old room
        if let Some(old_room) = rooms.get_mut(&old_room_name) {
            old_room.remove_member(nickname);

            // Notify old room
            let msg = Message::system(format!("{} left the room", nickname));
            old_room.broadcast(msg);

            // Clean up empty rooms (except lobby)
            if old_room.is_empty() && old_room_name != LOBBY_ROOM {
                rooms.remove(&old_room_name);
            }
        }

        // Create room if doesn't exist
        if !rooms.contains_key(&room_name) {
            rooms.insert(room_name.clone(), Room::new(room_name.clone()));
        }

        // Add to new room
        if let Some(new_room) = rooms.get_mut(&room_name) {
            new_room.add_member(nickname.to_string());

            // Notify new room
            let msg = Message::system(format!("{} joined the room", nickname));
            new_room.broadcast(msg);
        }

        // Update user's current room
        user_info.current_room = room_name;

        Ok(())
    }

    /// Get list of all rooms with member counts
    pub async fn list_rooms(&self) -> Vec<(String, usize)> {
        let rooms = self.rooms.read().await;
        rooms
            .iter()
            .map(|(name, room)| (name.clone(), room.member_count()))
            .collect()
    }

    /// Get list of users in a specific room
    pub async fn list_room_users(&self, room_name: &str) -> Vec<String> {
        let rooms = self.rooms.read().await;
        rooms
            .get(room_name)
            .map(|room| room.members.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Subscribe to messages in a room
    pub async fn subscribe_to_room(&self, room_name: &str) -> Option<tokio::sync::broadcast::Receiver<Message>> {
        let rooms = self.rooms.read().await;
        rooms.get(room_name).map(|room| room.subscribe())
    }

    /// Broadcast message to a room
    pub async fn broadcast_to_room(&self, room_name: &str, message: Message) {
        let rooms = self.rooms.read().await;
        if let Some(room) = rooms.get(room_name) {
            room.broadcast(message);
        }
    }

    /// Send private message to a user
    pub async fn send_private_message(&self, to: &str, message: Message) -> Result<(), String> {
        let users = self.users.read().await;
        let user_info = users.get(to).ok_or("User not found".to_string())?;

        user_info
            .tx
            .send(message)
            .map_err(|_| "Failed to send message".to_string())?;

        Ok(())
    }

    /// Get current room for a user
    pub async fn get_user_room(&self, nickname: &str) -> Option<String> {
        let users = self.users.read().await;
        users.get(nickname).map(|info| info.current_room.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_user() {
        let server = ChatServer::new(10);
        let (tx, _rx) = mpsc::unbounded_channel();

        let result = server.register_user("Alice".to_string(), tx).await;
        assert!(result.is_ok());

        assert!(server.is_nickname_taken("Alice").await);
        assert!(!server.is_nickname_taken("Bob").await);
    }

    #[tokio::test]
    async fn test_duplicate_nickname() {
        let server = ChatServer::new(10);
        let (tx1, _rx1) = mpsc::unbounded_channel();
        let (tx2, _rx2) = mpsc::unbounded_channel();

        server.register_user("Alice".to_string(), tx1).await.unwrap();
        let result = server.register_user("Alice".to_string(), tx2).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_change_nickname() {
        let server = ChatServer::new(10);
        let (tx, _rx) = mpsc::unbounded_channel();

        server.register_user("Alice".to_string(), tx).await.unwrap();

        let result = server.change_nickname("Alice", "AliceNew".to_string()).await;
        assert!(result.is_ok());

        assert!(!server.is_nickname_taken("Alice").await);
        assert!(server.is_nickname_taken("AliceNew").await);
    }

    #[tokio::test]
    async fn test_join_room() {
        let server = ChatServer::new(10);
        let (tx, _rx) = mpsc::unbounded_channel();

        server.register_user("Alice".to_string(), tx).await.unwrap();

        // Join new room
        server.join_room("Alice", "rust-chat".to_string()).await.unwrap();

        let room = server.get_user_room("Alice").await;
        assert_eq!(room, Some("rust-chat".to_string()));

        // Check rooms list
        let rooms = server.list_rooms().await;
        assert!(rooms.iter().any(|(name, _)| name == "rust-chat"));
    }

    #[tokio::test]
    async fn test_unregister_user() {
        let server = ChatServer::new(10);
        let (tx, _rx) = mpsc::unbounded_channel();

        server.register_user("Alice".to_string(), tx).await.unwrap();
        server.unregister_user("Alice").await;

        assert!(!server.is_nickname_taken("Alice").await);
    }
}
