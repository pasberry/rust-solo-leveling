use crate::messages::{ClientMessage, ServerMessage};
use crate::state::{AppState, ClientId};
use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;

pub async fn handle_websocket(socket: WebSocket, state: AppState) {
    let client_id = Uuid::new_v4();
    tracing::info!("WebSocket client {} connected", client_id);

    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

    state.register_client(client_id, tx.clone()).await;

    let mut broadcast_rx = state.broadcast_tx.subscribe();

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    let client_id_clone = client_id;
    let state_clone = state.clone();
    let receive_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                        handle_client_message(client_msg, &client_id_clone, &state_clone, &tx).await;
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(e) => {
                    tracing::error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    let client_id_clone2 = client_id;
    let state_clone2 = state.clone();
    let broadcast_task = tokio::spawn(async move {
        while let Ok((channel, message)) = broadcast_rx.recv().await {
            let connections = state_clone2.connections.read().await;
            if let Some(client) = connections.get(&client_id_clone2) {
                if client.subscriptions.contains(&channel) {
                    let _ = client.tx.send(message);
                }
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = receive_task => {},
        _ = broadcast_task => {},
    }

    state.unregister_client(&client_id).await;
    tracing::info!("WebSocket client {} disconnected", client_id);
}

async fn handle_client_message(
    msg: ClientMessage,
    client_id: &ClientId,
    state: &AppState,
    tx: &mpsc::UnboundedSender<ServerMessage>,
) {
    match msg {
        ClientMessage::Subscribe { channel } => {
            if state.subscribe(client_id, channel.clone()).await {
                let _ = tx.send(ServerMessage::Subscribed { channel });
            } else {
                let _ = tx.send(ServerMessage::Error {
                    message: "Failed to subscribe".to_string(),
                });
            }
        }
        ClientMessage::Unsubscribe { channel } => {
            if state.unsubscribe(client_id, &channel).await {
                let _ = tx.send(ServerMessage::Unsubscribed { channel });
            }
        }
        ClientMessage::Ping => {
            let _ = tx.send(ServerMessage::Pong);
        }
    }
}
