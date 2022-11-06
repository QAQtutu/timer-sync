use crate::msg::parse_client_message;
use crate::{msg, server};
use actix::{
    fut, Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, ContextFutureSpawner, Handler,
    StreamHandler, WrapFuture,
};
use actix_web_actors::ws;
use log::error;

#[derive(Debug)]
pub struct Client {
    pub sid: String,
    pub cid: u64,
    pub ip: String,
    pub server: Addr<server::Server>,
}

impl Actor for Client {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();

        self.server
            .send(msg::Connect {
                ip: self.ip.to_string(),
                sid: self.sid.clone(),
                cid: self.cid,
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, _act, ctx| {
                match res {
                    Ok(true) => {}
                    // something is wrong with timer server
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Client {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        match msg {
            ws::Message::Text(text) => {
                let msg = parse_client_message(text.trim());
                if let Ok(msg) = msg {
                    let wapper = msg::ClientMessageWapper {
                        ip: self.ip.clone(),
                        sid: self.sid.clone(),
                        cid: self.cid,
                        msg,
                    };
                    self.server.do_send(wapper);
                };
            }
            _ => {
                error!("Unsupported message type.")
            }
        }
    }
}

impl Handler<msg::ServerMessage> for Client {
    type Result = ();

    fn handle(&mut self, msg: msg::ServerMessage, ctx: &mut Self::Context) {
        let json = msg::format_server_message(msg);
        if let Ok(json) = json {
            ctx.text(json);
        }
    }
}
