use actix_web::{web, HttpRequest, Responder};
use actix_ws::Message;
use futures::StreamExt;
use std::time::Duration;

pub async fn ws_route(req: HttpRequest, body: web::Payload) -> actix_web::Result<impl Responder> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    // HEARTBEAT TASK
    let mut hb_session = session.clone();
    actix_web::rt::spawn(async move {
        loop {
            actix_web::rt::time::sleep(Duration::from_secs(5)).await;

            // send ping or stop if connection closed
            if hb_session.ping(b"").await.is_err() {
                break;
            }
        }
    });

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        return;
                    }
                }
                Message::Pong(_) => {
                    trace!("Websocket received pong");
                }
                Message::Text(msg) => println!("Got text: {msg}"),
                _ => break,
            }
        }

        let _ = session.close(None).await;
    });

    Ok(response)
}
