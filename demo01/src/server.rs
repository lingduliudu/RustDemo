use actix::prelude::*;
use std::collections::HashMap;

pub struct ChatServer {
    sessions: HashMap<usize, Recipient<ServerMessage>>,
    counter: usize,
}

/**************************************************************
* Description: 定义一个广播消息
* Author: yuanhao
* Versions: V1
**************************************************************/
#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastMessage {
    pub msg: String,
}

impl Handler<BroadcastMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, _ctx: &mut Context<Self>) {
        self.broadcast(msg.msg);
    }
}

impl ChatServer {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            counter: 0,
        }
    }

    /// 供外部（API）直接调用，进行广播
    pub fn broadcast(&self, text: String) {
        for (_id, addr) in &self.sessions {
            let _ = addr.do_send(ServerMessage(text.clone()));
        }
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

/// Session 注册
#[derive(Message)]
#[rtype(result = "usize")]
pub struct Connect {
    pub id: usize,
    pub addr: Recipient<ServerMessage>,
}

/// Session 离开
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

/// Session 发消息到服务器（广播给所有人）
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub id: usize,
    pub msg: String,
}

/// 服务器发消息到 Session
#[derive(Message)]
#[rtype(result = "()")]
pub struct ServerMessage(pub String);

impl Handler<Connect> for ChatServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        self.counter += 1;
        self.sessions.insert(msg.id, msg.addr);
        msg.id
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        self.sessions.remove(&msg.id);
    }
}

impl Handler<ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        let text = format!("User {}: {}", msg.id, msg.msg);
        let x = &self.sessions.get(&msg.id);
        match x {
            Some(addr) => {
                addr.do_send(ServerMessage(text.clone()));
            }
            None => {
                // 未找到链接已经断开/关闭
            }
        }
    }
}
