use actix::prelude::*;
use std::collections::HashMap;

/**************************************************************
* 对话服务要求规则:
* 1. 想要处理必须先定义一条消息类型 然后使用do_send(消息类型) 进行消息通话和调用
***************************************************************
*/

/**************************************************************
* Description: 对话中心服务直接发送的消息
* Author: yuanhao
* Versions: V1
**************************************************************/
#[derive(Message)]
#[rtype(result = "()")]
pub struct ServerMessage(pub String);

/**************************************************************
* Description: 对话服务中心
* Author: yuanhao
* Versions: V1
**************************************************************/
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

/**************************************************************
* Description: 定义广播消息处理逻辑
* Author: yuanhao
* Versions: V1
**************************************************************/
impl Handler<BroadcastMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, _ctx: &mut Context<Self>) {
        self.broadcast(msg.msg);
    }
}

/**************************************************************
* Description: 定义对话服务中心独属方法
* Author: yuanhao
* Versions: V1
**************************************************************/
impl ChatServer {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            counter: 0,
        }
    }
    pub fn broadcast(&self, text: String) {
        for (_id, addr) in &self.sessions {
            let _ = addr.do_send(ServerMessage(text.clone()));
        }
    }
}

/**************************************************************
* Description: 实现Actor 对话中心服务
* Author: yuanhao
* Versions: V1
**************************************************************/
impl Actor for ChatServer {
    type Context = Context<Self>;
}

/**************************************************************
* Description: 连接时消息
* Author: yuanhao
* Versions: V1
**************************************************************/
#[derive(Message)]
#[rtype(result = "usize")]
pub struct Connect {
    pub id: usize,
    pub addr: Recipient<ServerMessage>,
}
/**************************************************************
* Description: 连接时消息的处理
* Author: yuanhao
* Versions: V1
**************************************************************/
impl Handler<Connect> for ChatServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        self.counter += 1;
        self.sessions.insert(msg.id, msg.addr);
        msg.id
    }
}

/**************************************************************
* Description: 断开连接时消息
* Author: yuanhao
* Versions: V1
**************************************************************/
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}
/**************************************************************
* Description: 连接断开时消息的处理
* Author: yuanhao
* Versions: V1
**************************************************************/
impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        self.sessions.remove(&msg.id);
    }
}

/**************************************************************
* Description: 1 vs 1 消息 通过对话中心通信
* Author: yuanhao
* Versions: V1
**************************************************************/
#[derive(Message)]
#[rtype(result = "(i32)")]
pub struct ClientMessage {
    pub id: usize,
    pub msg: String,
}
/**************************************************************
* Description: 1 vs 1 消息 通过对话中心处理逻辑
* Author: yuanhao
* Versions: V1
**************************************************************/
impl Handler<ClientMessage> for ChatServer {
    type Result = i32;
    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) -> Self::Result {
        let text = format!("User {}: {}", msg.id, msg.msg);
        let x = &self.sessions.get(&msg.id);
        match x {
            Some(addr) => {
                addr.do_send(ServerMessage(text.clone()));
                return 1;
            }
            None => {
                // 未找到链接已经断开/关闭
                return 0;
            }
        }
    }
}
