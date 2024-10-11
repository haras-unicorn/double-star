use futures::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
pub async fn run(
  double_star_tx: flume::Sender<gravity::DoubleStarMessage>,
  orbitus_rx: flume::Receiver<gravity::OrbitusMessage>,
  config: super::config::Config,
) -> anyhow::Result<()> {
  let websocket_host = config.websocket.host;
  let websocket_port = config.websocket.port;
  let websocket_protocol = if config.websocket.ssl { "wss" } else { "ws" };

  let (socket, _) = connect_async(format!(
    "{websocket_protocol}://{websocket_host}:{websocket_port}/api/ws"
  ))
  .await?;

  let (mut socket_tx, mut socket_rx) = socket.split();

  let recv_handle = tokio::spawn(async move {
    while let Some(Ok(message)) = socket_rx.next().await {
      if let Message::Text(text) = message {
        if let Ok(message) =
          serde_json::de::from_str::<gravity::DoubleStarMessage>(text.as_str())
        {
          if let Err(err) = double_star_tx.send_async(message).await {
            tracing::error!("Failed forwarding message {}", err);
          }
        }
      }
    }
  });

  let send_handle = tokio::spawn(async move {
    while let Ok(message) = orbitus_rx.recv_async().await {
      let should_break = matches!(message, gravity::OrbitusMessage::Exited);
      if let Ok(message) = serde_json::ser::to_string(&message) {
        if let Err(err) = socket_tx.send(Message::Text(message)).await {
          tracing::error!("Failed forwarding message {}", err);
        }
      }
      if should_break {
        if let Err(err) = socket_tx.close().await {
          tracing::error!("Failed closing websocket {}", err);
        }
        break;
      }
    }
  });

  if let (_, Err(err)) | (Err(err), _) = tokio::join!(send_handle, recv_handle)
  {
    tracing::error!("Failed closing websocket {}", err)
  };

  Ok(())
}
