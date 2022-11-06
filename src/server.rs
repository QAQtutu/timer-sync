use std::collections::HashMap;

use actix::{Actor, Context, Handler};
use log::info;

use crate::msg::{self, ClientMessage, Connect, ServerMessage, StateSync, TimeSync};

pub struct Server {
    all_timer: HashMap<u64, Timer>,
    _count: u64,
}

impl Actor for Server {
    type Context = Context<Self>;
}

impl Server {
    pub fn new() -> Self {
        let mut all_timer = HashMap::new();
        let default_ct = Timer::new();
        all_timer.insert(0, default_ct);
        Server {
            all_timer,
            _count: 0u64,
        }
    }
}

type Session = actix::Recipient<ServerMessage>;

#[derive(Debug)]
struct Timer {
    counter: u64,
    time: u64,
    state: TimerState,
    pub all_users: HashMap<String, Session>,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            counter: 60 * 1000,
            time: 0,
            state: TimerState::STOP,
            all_users: HashMap::new(),
        }
    }

    pub fn start(&mut self) {
        if self.state == TimerState::START {
            return;
        }
        let now = now();
        self.time = now + self.counter;
        self.state = TimerState::START;

        self.state_sync_all();
    }

    pub fn stop(&mut self) {
        if self.state == TimerState::STOP {
            return;
        }
        let now = now();
        let mut diff = self.time as i64 - now as i64;
        if diff < 0 {
            diff = 0;
        }
        self.counter = diff as u64;
        self.state = TimerState::STOP;
        self.state_sync_all();
    }

    pub fn set_time(&mut self, time: u64) {
        self.state = TimerState::STOP;
        self.counter = time;
        self.state_sync_all();
    }

    pub fn time_sync(&self, sid: &str, time: u64) {
        self.send(
            sid,
            msg::ServerMessage::TimeSync(TimeSync {
                start: time,
                server: now(),
            }),
        )
    }

    pub fn state_sync(&self, sid: &str) {
        self.send(sid, self.build_state_sync_msg());
    }
    pub fn state_sync_all(&self) {
        self.send_all(self.build_state_sync_msg());
    }

    fn build_state_sync_msg(&self) -> msg::ServerMessage {
        let count = self.all_users.keys().len();
        return msg::ServerMessage::StateSync(StateSync {
            count_sessions: count,
            state: self.state.to_string(),
            time: self.time,
            counter: self.counter,
        });
    }

    pub fn send(&self, sid: &str, msg: msg::ServerMessage) {
        let session = self.all_users.get(sid);
        if let Some(session) = session {
            session.do_send(msg);
        }
    }
    pub fn send_all(&self, msg: msg::ServerMessage) {
        self.all_users
            .values()
            .for_each(|session| session.do_send(msg.clone()));
    }
}

#[derive(PartialEq, Debug)]
enum TimerState {
    START,
    STOP,
}

impl ToString for TimerState {
    fn to_string(&self) -> String {
        match self {
            TimerState::START => String::from("START"),
            TimerState::STOP => String::from("STOP"),
        }
    }
}

impl Handler<Connect> for Server {
    type Result = bool;

    fn handle(&mut self, msg: msg::Connect, _: &mut Context<Self>) -> Self::Result {
        let ct = self.all_timer.get_mut(&msg.cid);
        let ct = match ct {
            Some(ct) => ct,
            None => return false,
        };
        ct.all_users.insert(msg.sid, msg.addr);
        info!("Ip:{} add to room.", msg.ip);
        true
    }
}

impl Handler<msg::ClientMessageWapper> for Server {
    type Result = ();

    fn handle(&mut self, msg: msg::ClientMessageWapper, _: &mut Context<Self>) -> Self::Result {
        let ct = self.all_timer.get_mut(&msg.cid);
        let ct = match ct {
            Some(ct) => ct,
            None => return (),
        };
        match msg.msg {
            ClientMessage::TimeSync(time) => ct.time_sync(msg.sid.as_str(), time),
            ClientMessage::Start => {
                ct.start();
                info!("Ip:{} set start.", msg.ip);
            }
            ClientMessage::Stop => {
                ct.stop();
                info!("Ip:{} set stop.", msg.ip);
            }
            ClientMessage::StateSync => ct.state_sync(msg.sid.as_str()),
            ClientMessage::SetTime(time) => {
                ct.set_time(time);
                info!("Ip:{} set time to {}ms", msg.ip, time);
            }
            ClientMessage::None => {}
        }
        ()
    }
}

fn now() -> u64 {
    return chrono::Utc::now().timestamp_millis() as u64;
}
