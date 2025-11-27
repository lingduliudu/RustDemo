use actix::prelude::*;
use actix_web_actors::ws;

use crate::server::*;

pub struct WsSession {
    pub id: usize,
    pub server: Addr<ChatServer>,
}

impl WsSession {
    pub fn new(server: Addr<ChatServer>) -> Self {
        Self { id: 0, server }
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address().recipient();
        self.server
            .send(Connect { addr })
            .into_actor(self)
            .then(|res, act, _ctx| {
                if let Ok(id) = res {
                    act.id = id;
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        self.server.do_send(Disconnect { id: self.id });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut ws::WebsocketContext<Self>,
    ) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                self.server.do_send(ClientMessage {
                    id: self.id,
                    msg: text.to_string(),
                });
            }
            Ok(ws::Message::Ping(s)) => ctx.pong(&s),
            _ => {}
        }
    }
}

impl Handler<ServerMessage> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: ServerMessage, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.text(msg.0);
    }
}
