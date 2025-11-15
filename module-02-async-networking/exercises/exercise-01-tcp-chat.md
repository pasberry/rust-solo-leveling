# Exercise 01: Multi-Room TCP Chat Server

## Objective

Build a production-quality TCP chat server with multiple chat rooms, user nicknames, and message broadcasting.

**Estimated time:** 3-4 hours

## Requirements

### Core Features

1. **User Connection**
   - Accept TCP connections on port 8080
   - Prompt new users for a nickname
   - Validate nicknames (alphanumeric, 3-20 characters, unique)
   - Welcome message showing available commands

2. **Chat Rooms**
   - Default "lobby" room (everyone starts here)
   - Users can create new rooms
   - Users can join/leave rooms
   - Users can list all rooms and see user counts
   - Private messages between users

3. **Message Broadcasting**
   - Messages broadcast to all users in the same room
   - Show sender's nickname and timestamp
   - Support for commands (prefixed with `/`)
   - Notify room when users join/leave

4. **Commands**
   - `/nick <name>` - Change nickname
   - `/join <room>` - Join a room (creates if doesn't exist)
   - `/leave` - Leave current room (return to lobby)
   - `/rooms` - List all rooms with user counts
   - `/users` - List users in current room
   - `/msg <user> <message>` - Send private message
   - `/quit` - Disconnect from server

### Technical Requirements

1. **Concurrency**
   - Handle 100+ concurrent connections
   - No blocking operations in async code
   - Proper use of channels for communication

2. **Error Handling**
   - Graceful handling of client disconnects
   - Invalid command responses
   - Connection errors don't crash server

3. **Resource Management**
   - Clean up disconnected users
   - Remove empty rooms
   - Limit maximum connections (configurable)

## Project Structure

```
exercise-01-tcp-chat/
├── Cargo.toml
└── src/
    ├── main.rs          # Server setup and main loop
    ├── server.rs        # ChatServer state management
    ├── client.rs        # Client connection handler
    ├── room.rs          # Room management
    └── message.rs       # Message types and parsing
```

## Example Session

```
$ telnet localhost 8080
Connected to localhost.
Enter your nickname: Alice
Welcome Alice! You are in #lobby
Type /help for available commands

Alice: Hello everyone!
[10:30:45] Alice: Hello everyone!

Bob joined #lobby
[10:30:50] *** Bob joined the room

Alice: /join rust-chat
You joined #rust-chat

Alice: /rooms
Available rooms:
  #lobby (2 users)
  #rust-chat (1 user)

Alice: /users
Users in #rust-chat:
  Alice (you)

Alice: /msg Bob Hey!
[Private to Bob]: Hey!

Alice: /quit
Goodbye!
```

## Testing Checklist

- [ ] Server starts and listens on port 8080
- [ ] Multiple clients can connect simultaneously
- [ ] Nicknames are validated and must be unique
- [ ] Messages are broadcast only to users in the same room
- [ ] Users can create and join rooms
- [ ] `/rooms` command shows accurate counts
- [ ] Private messages work correctly
- [ ] Server handles client disconnects gracefully
- [ ] Empty rooms are cleaned up
- [ ] Invalid commands return helpful error messages

## Hints

1. **State Management**: Use `Arc<RwLock<HashMap<...>>>` for shared state
2. **Broadcasting**: Use `tokio::sync::broadcast` for room messages
3. **Framing**: Use `BufReader` and `lines()` for message framing
4. **Commands**: Parse commands in a dedicated function
5. **Cleanup**: Use RAII pattern - clean up when client handler exits

## Bonus Challenges

1. **Persistence**: Save chat history to disk
2. **Authentication**: Add password-protected rooms
3. **Rate Limiting**: Prevent message spam (max 10 msg/sec per user)
4. **Message History**: Send last 10 messages when joining a room
5. **Admin Commands**: `/kick <user>`, `/ban <user>`

## Submission

Your solution should:
- Compile without warnings
- Handle all required commands
- Pass the testing checklist
- Include comments explaining design decisions
- Have no unwrap() in production code (use proper error handling)

Good luck! Remember: this exercise tests your understanding of async Rust, channels, and concurrent state management.
