use crate::message::Message;
use std::collections::HashSet;
use tokio::sync::broadcast;

/// Maximum number of messages buffered in room channel
const ROOM_CHANNEL_SIZE: usize = 100;

/// A chat room that broadcasts messages to all members
pub struct Room {
    pub name: String,
    pub members: HashSet<String>,
    pub tx: broadcast::Sender<Message>,
}

impl Room {
    pub fn new(name: String) -> Self {
        let (tx, _rx) = broadcast::channel(ROOM_CHANNEL_SIZE);

        Room {
            name,
            members: HashSet::new(),
            tx,
        }
    }

    /// Add a user to this room
    pub fn add_member(&mut self, nickname: String) {
        self.members.insert(nickname);
    }

    /// Remove a user from this room
    pub fn remove_member(&mut self, nickname: &str) {
        self.members.remove(nickname);
    }

    /// Check if room is empty
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// Get number of members
    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    /// Broadcast a message to all members in the room
    pub fn broadcast(&self, message: Message) {
        // Ignore error if no receivers (empty room)
        let _ = self.tx.send(message);
    }

    /// Subscribe to room messages
    pub fn subscribe(&self) -> broadcast::Receiver<Message> {
        self.tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_creation() {
        let room = Room::new("test".to_string());
        assert_eq!(room.name, "test");
        assert!(room.is_empty());
        assert_eq!(room.member_count(), 0);
    }

    #[test]
    fn test_add_remove_members() {
        let mut room = Room::new("test".to_string());

        room.add_member("Alice".to_string());
        assert_eq!(room.member_count(), 1);
        assert!(!room.is_empty());

        room.add_member("Bob".to_string());
        assert_eq!(room.member_count(), 2);

        room.remove_member("Alice");
        assert_eq!(room.member_count(), 1);

        room.remove_member("Bob");
        assert!(room.is_empty());
    }

    #[tokio::test]
    async fn test_broadcast() {
        let room = Room::new("test".to_string());
        let mut rx = room.subscribe();

        let msg = Message::system("Test message".to_string());
        room.broadcast(msg.clone());

        let received = rx.recv().await.unwrap();
        match received {
            Message::System(content) => assert_eq!(content, "Test message"),
            _ => panic!("Expected System message"),
        }
    }
}
