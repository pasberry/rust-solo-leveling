# Exercise 03: Real-Time Notification System with WebSockets

## Objective

Build a real-time notification system using WebSockets that allows clients to subscribe to events and receive live updates.

**Estimated time:** 3-4 hours

## Requirements

### Core Features

1. **WebSocket Connections**
   - Clients connect via WebSocket
   - Support 1000+ concurrent connections
   - Automatic reconnection handling
   - Ping/pong for connection health

2. **Event Broadcasting**
   - Publish events to all connected clients
   - Subscribe to specific event channels
   - Unsubscribe from channels
   - Message filtering by channel

3. **HTTP API**
   - REST endpoint to publish events
   - List active connections
   - Get connection statistics
   - Health check

4. **Event Types**
   - System notifications
   - User-specific messages
   - Broadcast announcements
   - Custom events with payloads

### Technical Requirements

1. **Framework**: Axum with WebSocket support
2. **Concurrency**: Handle concurrent WebSocket connections
3. **Broadcasting**: Efficient message distribution
4. **State Management**: Track active connections and subscriptions
5. **Error Handling**: Graceful connection failures

## Architecture

```
┌─────────────────┐
│  HTTP Clients   │  POST /api/events (publish events)
└────────┬────────┘
         │
    ┌────▼────────────────┐
    │   HTTP API Server   │
    └────┬────────────────┘
         │
    ┌────▼────────────────┐
    │   Event Broadcast   │
    │      Channel        │
    └────┬────────────────┘
         │
  ┌──────┴──────────┐
  │                 │
┌─▼──────────┐  ┌──▼──────────┐
│ WebSocket  │  │ WebSocket   │
│ Client 1   │  │ Client 2    │
└────────────┘  └─────────────┘
```

## Message Protocol

### Client → Server

```json
{
  "type": "subscribe",
  "channel": "notifications"
}

{
  "type": "unsubscribe",
  "channel": "notifications"
}

{
  "type": "ping"
}
```

### Server → Client

```json
{
  "type": "event",
  "channel": "notifications",
  "data": {
    "id": "evt_123",
    "timestamp": "2024-01-15T10:30:00Z",
    "message": "New notification"
  }
}

{
  "type": "subscribed",
  "channel": "notifications"
}

{
  "type": "error",
  "message": "Invalid channel"
}

{
  "type": "pong"
}
```

## API Endpoints

### WebSocket Connection
```
WS  /ws
```

### Publish Event
```bash
POST /api/events
Content-Type: application/json

{
  "channel": "notifications",
  "data": {
    "message": "Hello, World!"
  }
}
```

### Get Statistics
```bash
GET /api/stats

Response:
{
  "active_connections": 42,
  "channels": {
    "notifications": 10,
    "alerts": 5,
    "updates": 27
  }
}
```

## Project Structure

```
exercise-03-websocket/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── websocket.rs      # WebSocket handler
    ├── broadcast.rs      # Event broadcasting
    ├── messages.rs       # Message types
    └── state.rs          # Connection state
```

## Example Usage

### JavaScript Client

```javascript
const ws = new WebSocket('ws://localhost:3000/ws');

ws.onopen = () => {
    // Subscribe to channel
    ws.send(JSON.stringify({
        type: 'subscribe',
        channel: 'notifications'
    }));
};

ws.onmessage = (event) => {
    const message = JSON.parse(event.data);
    console.log('Received:', message);

    if (message.type === 'event') {
        console.log('Event on', message.channel, ':', message.data);
    }
};

ws.onerror = (error) => {
    console.error('WebSocket error:', error);
};

ws.onclose = () => {
    console.log('Disconnected');
};
```

### Publish Event via HTTP

```bash
curl -X POST http://localhost:3000/api/events \
  -H "Content-Type: application/json" \
  -d '{
    "channel": "notifications",
    "data": {
      "message": "New update available!",
      "priority": "high"
    }
  }'
```

## Testing Checklist

- [ ] WebSocket connections accepted
- [ ] Multiple clients can connect simultaneously
- [ ] Subscribe/unsubscribe messages work
- [ ] Events broadcast to all subscribed clients
- [ ] Clients only receive events from subscribed channels
- [ ] Disconnected clients are cleaned up
- [ ] HTTP publish endpoint works
- [ ] Statistics endpoint returns accurate counts
- [ ] Ping/pong keeps connections alive
- [ ] Error messages sent for invalid requests

## Bonus Challenges

1. **Authentication**: Add token-based authentication for WebSocket connections
2. **Message Queue**: Add Redis pub/sub for multi-server deployment
3. **Presence**: Track which users are online
4. **Private Channels**: User-specific channels
5. **Message History**: Send last N messages on subscribe
6. **Rate Limiting**: Limit messages per client
7. **Compression**: Use per-message deflate extension
8. **Metrics**: Track message rates, latencies
9. **Binary Messages**: Support binary (protobuf, msgpack)
10. **Room System**: Hierarchical channels (chat.room1, chat.room2)

## Hints

1. **State Management**: Use `Arc<RwLock<HashMap>>` for connection tracking
2. **Broadcasting**: Use `tokio::sync::broadcast` for pub/sub
3. **WebSocket Split**: Use `socket.split()` to handle send/receive separately
4. **Cleanup**: Use `select!` to detect disconnection
5. **Error Handling**: Don't crash server if one client has errors

## Comparison with Other Technologies

| Technology | Use Case |
|------------|----------|
| WebSockets | Full-duplex, low latency, bi-directional |
| Server-Sent Events (SSE) | Server→Client only, simpler |
| Long Polling | Fallback for old browsers |
| gRPC Streaming | Typed, efficient, complex setup |

## Resources

- [MDN WebSocket API](https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API)
- [Axum WebSocket Example](https://github.com/tokio-rs/axum/tree/main/examples/websockets)
- [WebSocket Protocol RFC 6455](https://tools.ietf.org/html/rfc6455)

Good luck!
