use crate::message::{parse_input, validate_nickname, Command, Message, HELP_TEXT};
use crate::server::ChatServer;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc;

/// Handle a client connection
pub async fn handle_client(socket: TcpStream, server: Arc<ChatServer>) {
    let addr = socket.peer_addr().unwrap();
    println!("New connection from {}", addr);

    let (reader, mut writer) = socket.into_split();
    let reader = BufReader::new(reader);
    let mut lines = reader.lines();

    // Prompt for nickname
    if writer
        .write_all(b"Welcome to the chat server!\nEnter your nickname: ")
        .await
        .is_err()
    {
        return;
    }

    // Read nickname
    let nickname = match lines.next_line().await {
        Ok(Some(line)) => line.trim().to_string(),
        _ => {
            eprintln!("Failed to read nickname from {}", addr);
            return;
        }
    };

    // Validate nickname
    if let Err(e) = validate_nickname(&nickname) {
        let _ = writer
            .write_all(format!("Invalid nickname: {}\n", e).as_bytes())
            .await;
        return;
    }

    // Create channel for outgoing messages to this client
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Register user
    if let Err(e) = server.register_user(nickname.clone(), tx.clone()).await {
        let _ = writer
            .write_all(format!("Registration failed: {}\n", e).as_bytes())
            .await;
        return;
    }

    println!("{} ({}) connected", nickname, addr);

    // Send welcome message
    let welcome = format!(
        "Welcome {}! You are in #lobby\nType /help for available commands\n",
        nickname
    );
    if writer.write_all(welcome.as_bytes()).await.is_err() {
        server.unregister_user(&nickname).await;
        return;
    }

    // Subscribe to lobby messages
    let current_room = "lobby".to_string();
    let mut room_rx = server
        .subscribe_to_room(&current_room)
        .await
        .expect("Lobby should exist");

    // Notify lobby
    let join_msg = Message::system(format!("{} joined the room", nickname));
    server.broadcast_to_room(&current_room, join_msg).await;

    // Main client loop
    let nickname_clone = nickname.clone();
    let server_clone = server.clone();

    loop {
        select! {
            // Incoming messages from user
            line = lines.next_line() => {
                match line {
                    Ok(Some(line)) => {
                        let line = line.trim();
                        if line.is_empty() {
                            continue;
                        }

                        // Try to parse as command
                        match parse_input(line) {
                            Ok(cmd) => {
                                if !handle_command(
                                    cmd,
                                    &nickname_clone,
                                    &server_clone,
                                    &mut writer,
                                    &mut room_rx,
                                ).await {
                                    // Quit command
                                    break;
                                }
                            }
                            Err(_) => {
                                // Not a command, treat as regular message
                                let current_room = server_clone.get_user_room(&nickname_clone).await;
                                if let Some(room_name) = current_room {
                                    let msg = Message::chat(nickname_clone.clone(), line.to_string());
                                    server_clone.broadcast_to_room(&room_name, msg).await;
                                }
                            }
                        }
                    }
                    Ok(None) | Err(_) => {
                        // Connection closed
                        break;
                    }
                }
            }

            // Outgoing messages to user (from room or private)
            msg = rx.recv() => {
                match msg {
                    Some(msg) => {
                        let formatted = format!("{}\n", msg.format());
                        if writer.write_all(formatted.as_bytes()).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }

            // Broadcast messages from current room
            msg = room_rx.recv() => {
                match msg {
                    Ok(msg) => {
                        let formatted = format!("{}\n", msg.format());
                        if writer.write_all(formatted.as_bytes()).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => {
                        // Channel closed or lagged
                    }
                }
            }
        }
    }

    // Cleanup
    println!("{} ({}) disconnected", nickname, addr);
    server.unregister_user(&nickname).await;

    let _ = writer.write_all(b"Goodbye!\n").await;
}

/// Handle a command from the user
/// Returns false if user wants to quit
async fn handle_command(
    cmd: Command,
    nickname: &str,
    server: &ChatServer,
    writer: &mut tokio::net::tcp::OwnedWriteHalf,
    room_rx: &mut tokio::sync::broadcast::Receiver<Message>,
) -> bool {
    match cmd {
        Command::Nick(new_nick) => {
            // Validate new nickname
            if let Err(e) = validate_nickname(&new_nick) {
                let msg = Message::error(e);
                let _ = writer.write_all(format!("{}\n", msg.format()).as_bytes()).await;
                return true;
            }

            // Change nickname
            match server.change_nickname(nickname, new_nick.clone()).await {
                Ok(()) => {
                    let msg = format!("You are now known as {}\n", new_nick);
                    let _ = writer.write_all(msg.as_bytes()).await;
                }
                Err(e) => {
                    let msg = Message::error(e);
                    let _ = writer.write_all(format!("{}\n", msg.format()).as_bytes()).await;
                }
            }
        }

        Command::Join(room_name) => {
            match server.join_room(nickname, room_name.clone()).await {
                Ok(()) => {
                    let msg = format!("You joined #{}\n", room_name);
                    let _ = writer.write_all(msg.as_bytes()).await;

                    // Subscribe to new room
                    if let Some(new_rx) = server.subscribe_to_room(&room_name).await {
                        *room_rx = new_rx;
                    }
                }
                Err(e) => {
                    let msg = Message::error(e);
                    let _ = writer.write_all(format!("{}\n", msg.format()).as_bytes()).await;
                }
            }
        }

        Command::Leave => {
            // Return to lobby
            match server.join_room(nickname, "lobby".to_string()).await {
                Ok(()) => {
                    let msg = "You returned to #lobby\n";
                    let _ = writer.write_all(msg.as_bytes()).await;

                    // Subscribe to lobby
                    if let Some(new_rx) = server.subscribe_to_room("lobby").await {
                        *room_rx = new_rx;
                    }
                }
                Err(e) => {
                    let msg = Message::error(e);
                    let _ = writer.write_all(format!("{}\n", msg.format()).as_bytes()).await;
                }
            }
        }

        Command::Rooms => {
            let rooms = server.list_rooms().await;
            let mut output = "Available rooms:\n".to_string();
            for (name, count) in rooms {
                output.push_str(&format!("  #{} ({} users)\n", name, count));
            }
            let _ = writer.write_all(output.as_bytes()).await;
        }

        Command::Users => {
            if let Some(current_room) = server.get_user_room(nickname).await {
                let users = server.list_room_users(&current_room).await;
                let mut output = format!("Users in #{}:\n", current_room);
                for user in users {
                    if user == nickname {
                        output.push_str(&format!("  {} (you)\n", user));
                    } else {
                        output.push_str(&format!("  {}\n", user));
                    }
                }
                let _ = writer.write_all(output.as_bytes()).await;
            }
        }

        Command::Msg { recipient, content } => {
            let msg = Message::private(nickname.to_string(), recipient.clone(), content.clone());

            match server.send_private_message(&recipient, msg).await {
                Ok(()) => {
                    let confirmation = format!("[Private to {}]: {}\n", recipient, content);
                    let _ = writer.write_all(confirmation.as_bytes()).await;
                }
                Err(e) => {
                    let msg = Message::error(e);
                    let _ = writer.write_all(format!("{}\n", msg.format()).as_bytes()).await;
                }
            }
        }

        Command::Help => {
            let _ = writer.write_all(HELP_TEXT.as_bytes()).await;
        }

        Command::Quit => {
            return false;
        }
    }

    true
}
