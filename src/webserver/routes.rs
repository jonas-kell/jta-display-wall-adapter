use crate::aruco::aruco_image;
use crate::webserver::interface::MessageFromWebControl;
use crate::{aruco::aruco_bits, server::comm_channel::InstructionCommunicationChannel};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use actix_ws::Message;
use futures::StreamExt;
use std::{sync::Arc, time::Duration};

use super::MessageToWebControl;

pub async fn ws_route(
    comm_channel_data: web::Data<Arc<InstructionCommunicationChannel>>,
    req: HttpRequest,
    body: web::Payload,
) -> actix_web::Result<impl Responder> {
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

    // Sender task
    let mut sender_session = session.clone();
    let mut web_control_receiver = comm_channel_data.web_control_receiver();
    actix_web::rt::spawn(async move {
        loop {
            match web_control_receiver.wait_for_some_data().await {
                Err(_) => {
                    trace!("No new command to send to web control within timeout interval");
                    continue;
                }
                Ok(Err(e)) => {
                    error!("Error reading from internal channel: {}", e.to_string());
                    break;
                }
                Ok(Ok(msg)) => match msg {
                    MessageToWebControl::CurrentDisplayFrame(frame) => {
                        match sender_session.binary(frame).await {
                            Ok(()) => {
                                trace!("Binary Communication to web control was sent out");
                                continue;
                            }
                            Err(e) => {
                                warn!("Communication to web control went away while sending frame data: {}", e.to_string());
                                break;
                            }
                        }
                    }
                    other => {
                        match sender_session
                                .text(match serde_json::to_string(&other) {
                                    Ok(str) => str,
                                    Err(e) => {
                                        error!(
                                            "Serde could not serealize message. This should not happen: {}",
                                            e
                                        );
                                        break;
                                    }
                                })
                                .await
                            {
                                Ok(()) => {
                                    trace!("Communication to web control was sent out");
                                    continue;
                                }
                                Err(e) => {
                                    warn!(
                                        "Communication to web control went away while sending message: {}",
                                        e.to_string()
                                    );
                                    break;
                                }
                            }
                    }
                },
            }
        }
    });

    let comm_channel_data_write = comm_channel_data;
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
                Message::Text(msg) => {
                    let msg_parsed = match serde_json::from_str::<MessageFromWebControl>(&msg) {
                        Ok(m) => m,
                        Err(e) => {
                            error!(
                                "Could not parse a websocket message: {}\n{}",
                                msg,
                                e.to_string()
                            );

                            continue;
                        }
                    };

                    trace!("Websocket received message: {:?}", msg_parsed);
                    match comm_channel_data_write.take_in_command_from_web_control(msg_parsed) {
                        Ok(()) => trace!("Websocket message forwarded into internal communication"),
                        Err(e) => {
                            error!("Websocket could not forward into internal message queue. That one should be always empty though. Not recoverable error: {}",e);
                            break;
                        }
                    }
                }
                _ => break,
            }
        }

        let _ = session.close(None).await;
    });

    Ok(response)
}

pub async fn aruco_route(id: web::Path<u16>) -> actix_web::Result<impl Responder> {
    let id = id.into_inner();

    match serde_json::to_string(&aruco_bits(id.into())) {
        Ok(content) => Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(content)),
        Err(err) => Ok(HttpResponse::InternalServerError()
            .content_type("application/text")
            .body(err.to_string())),
    }
}

pub async fn aruco_image_route(path: web::Path<(u16, u16)>) -> actix_web::Result<impl Responder> {
    let (id, width) = path.into_inner();

    match aruco_image(id.into(), width.into()) {
        Ok(content) => Ok(HttpResponse::Ok()
            .insert_header(("Content-Disposition", "inline"))
            .content_type("image/png")
            .body(content)),
        Err(err) => Ok(HttpResponse::InternalServerError()
            .content_type("application/text")
            .body(err.to_string())),
    }
}
