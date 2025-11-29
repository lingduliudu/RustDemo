use crate::server::*;
use actix::prelude::*;
use actix_web_actors::ws;
use log::info;
use serde_json::Value;

/**************************************************************
* Description: websocket 连接基本配置
* Author: yuanhao
* Versions: V1
**************************************************************/
pub struct WsSession {
    pub id: usize,
    pub server: Addr<ChatServer>,
}

/**************************************************************
* Description: websocket 连接初始化配置
* Author: yuanhao
* Versions: V1
**************************************************************/
impl WsSession {
    pub fn new(id: usize, server: Addr<ChatServer>) -> Self {
        Self { id, server }
    }
}

/**************************************************************
* Description: websocket 连接时的连接/断开 配置
* Author: yuanhao
* Versions: V1
**************************************************************/
impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address().recipient();
        let client_id = self.id;
        self.server
            .send(Connect {
                id: client_id,
                addr,
            })
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

/**************************************************************
* Description: websocket 连接的消息处理逻辑
* Author: yuanhao
* Versions: V1
**************************************************************/
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut ws::WebsocketContext<Self>,
    ) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                // 判断是否是发给他人的消息
                match serde_json::from_str::<Value>(text.to_string().as_str()) {
                    Ok(v) => {
                        info!("测试消息原始数据:{}", v);
                        if !v["user_id"].is_null() {
                            self.server.do_send(ClientMessage {
                                id: v["user_id"].as_i64().unwrap() as usize,
                                msg: v["msg"].as_str().unwrap().to_string(),
                            });
                        }
                    }
                    Err(e) => {
                        info!("解析错误?{}", e);
                    }
                }
            }
            Ok(ws::Message::Ping(s)) => ctx.pong(&s),
            _ => {}
        }
    }
}

/**************************************************************
* Description:  对话服务中心发送消息给客户端处理逻辑
* Author: yuanhao
* Versions: V1
**************************************************************/
impl Handler<ServerMessage> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: ServerMessage, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.text(msg.0);
    }
}
