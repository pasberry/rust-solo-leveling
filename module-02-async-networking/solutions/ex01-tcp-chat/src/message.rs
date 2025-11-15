use chrono::Utc;

/// Message types that can be sent between server and clients
#[derive(Debug, Clone)]
pub enum Message {
    /// Regular chat message in a room
    Chat {
        sender: String,
        content: String,
        timestamp: chrono::DateTime<Utc>,
    },
    /// Private message between users
    Private {
        from: String,
        to: String,
        content: String,
        timestamp: chrono::DateTime<Utc>,
    },
    /// System notification (join, leave, etc.)
    System(String),
    /// Server error message
    Error(String),
}

impl Message {
    pub fn chat(sender: String, content: String) -> Self {
        Message::Chat {
            sender,
            content,
            timestamp: Utc::now(),
        }
    }

    pub fn private(from: String, to: String, content: String) -> Self {
        Message::Private {
            from,
            to,
            content,
            timestamp: Utc::now(),
        }
    }

    pub fn system(content: String) -> Self {
        Message::System(content)
    }

    pub fn error(content: String) -> Self {
        Message::Error(content)
    }

    /// Format message for display to client
    pub fn format(&self) -> String {
        match self {
            Message::Chat { sender, content, timestamp } => {
                format!("[{}] {}: {}", timestamp.format("%H:%M:%S"), sender, content)
            }
            Message::Private { from, content, timestamp, .. } => {
                format!("[{}] [Private from {}]: {}", timestamp.format("%H:%M:%S"), from, content)
            }
            Message::System(content) => {
                format!("*** {}", content)
            }
            Message::Error(content) => {
                format!("ERROR: {}", content)
            }
        }
    }
}

/// Commands that users can send
#[derive(Debug, Clone)]
pub enum Command {
    Nick(String),
    Join(String),
    Leave,
    Rooms,
    Users,
    Msg { recipient: String, content: String },
    Help,
    Quit,
}

/// Parse user input into a command or regular message
pub fn parse_input(input: &str) -> Result<Command, String> {
    let input = input.trim();

    if input.is_empty() {
        return Err("Empty message".to_string());
    }

    if !input.starts_with('/') {
        return Err("Not a command".to_string());
    }

    let parts: Vec<&str> = input[1..].splitn(2, ' ').collect();
    let command = parts[0].to_lowercase();

    match command.as_str() {
        "nick" => {
            if parts.len() < 2 {
                return Err("Usage: /nick <new_nickname>".to_string());
            }
            Ok(Command::Nick(parts[1].to_string()))
        }
        "join" => {
            if parts.len() < 2 {
                return Err("Usage: /join <room_name>".to_string());
            }
            Ok(Command::Join(parts[1].to_string()))
        }
        "leave" => Ok(Command::Leave),
        "rooms" => Ok(Command::Rooms),
        "users" => Ok(Command::Users),
        "msg" | "pm" => {
            if parts.len() < 2 {
                return Err("Usage: /msg <user> <message>".to_string());
            }
            let msg_parts: Vec<&str> = parts[1].splitn(2, ' ').collect();
            if msg_parts.len() < 2 {
                return Err("Usage: /msg <user> <message>".to_string());
            }
            Ok(Command::Msg {
                recipient: msg_parts[0].to_string(),
                content: msg_parts[1].to_string(),
            })
        }
        "help" => Ok(Command::Help),
        "quit" | "exit" => Ok(Command::Quit),
        _ => Err(format!("Unknown command: /{}. Type /help for available commands", command)),
    }
}

/// Validate nickname according to rules
pub fn validate_nickname(nick: &str) -> Result<(), String> {
    if nick.len() < 3 {
        return Err("Nickname must be at least 3 characters".to_string());
    }

    if nick.len() > 20 {
        return Err("Nickname must be at most 20 characters".to_string());
    }

    if !nick.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err("Nickname can only contain letters, numbers, underscore and dash".to_string());
    }

    Ok(())
}

pub const HELP_TEXT: &str = r#"
Available commands:
  /nick <name>        - Change your nickname
  /join <room>        - Join a room (creates if doesn't exist)
  /leave              - Leave current room (return to lobby)
  /rooms              - List all rooms with user counts
  /users              - List users in current room
  /msg <user> <text>  - Send private message
  /help               - Show this help
  /quit               - Disconnect from server

To send a message, just type it (no / prefix)
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_nickname() {
        assert!(validate_nickname("Alice").is_ok());
        assert!(validate_nickname("Bob123").is_ok());
        assert!(validate_nickname("user_name").is_ok());

        assert!(validate_nickname("ab").is_err());  // Too short
        assert!(validate_nickname("a".repeat(21).as_str()).is_err());  // Too long
        assert!(validate_nickname("alice!").is_err());  // Invalid char
    }

    #[test]
    fn test_parse_command() {
        match parse_input("/nick Alice").unwrap() {
            Command::Nick(name) => assert_eq!(name, "Alice"),
            _ => panic!("Expected Nick command"),
        }

        match parse_input("/join rust-chat").unwrap() {
            Command::Join(room) => assert_eq!(room, "rust-chat"),
            _ => panic!("Expected Join command"),
        }

        match parse_input("/msg Bob Hello there").unwrap() {
            Command::Msg { recipient, content } => {
                assert_eq!(recipient, "Bob");
                assert_eq!(content, "Hello there");
            }
            _ => panic!("Expected Msg command"),
        }

        assert!(parse_input("/unknown").is_err());
    }

    #[test]
    fn test_message_format() {
        let msg = Message::system("Test joined the room".to_string());
        assert!(msg.format().contains("***"));

        let msg = Message::error("Invalid command".to_string());
        assert!(msg.format().contains("ERROR:"));
    }
}
