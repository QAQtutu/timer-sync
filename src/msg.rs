use std::collections::HashMap;

use actix::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, from_value, Value};

#[derive(Message, Debug)]
#[rtype(bool)]
pub struct Connect {
    pub ip: String,
    pub sid: String,
    pub cid: u64,
    pub addr: Recipient<ServerMessage>,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub enum ServerMessage {
    TimeSync(TimeSync),
    StateSync(StateSync),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TimeSync {
    pub start: u64,
    pub server: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StateSync {
    pub count_sessions: usize,
    pub state: String,
    pub mode: String,
    pub time: u64,
    pub counter: u64,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub enum ClientMessage {
    TimeSync(u64),
    Start,
    Stop,
    StateSync,
    SetTime(u64),
    SetMode(String),
    None,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct ClientMessageWapper {
    pub ip: String,
    pub sid: String,
    pub cid: u64,
    pub msg: ClientMessage,
}

pub fn parse_client_message(msg: &str) -> Result<ClientMessage, serde_json::error::Error> {
    let mut value: Value = match from_str(msg) {
        Ok(value) => value,
        _ => return Ok(ClientMessage::None),
    };
    let ty = value["type"].take();
    let data = value["data"].take();

    match ty.as_str().unwrap_or("") {
        "TimeSync" => return Ok(ClientMessage::TimeSync(from_value(data)?)),
        "Start" => return Ok(ClientMessage::Start),
        "Stop" => return Ok(ClientMessage::Stop),
        "StateSync" => return Ok(ClientMessage::StateSync),
        "SetTime" => return Ok(ClientMessage::SetTime(from_value(data)?)),
        "SetMode" => return Ok(ClientMessage::SetMode(from_value(data)?)),
        _ => return Ok(ClientMessage::None),
    };
}

pub fn format_server_message(msg: ServerMessage) -> Result<String, serde_json::Error> {
    let mut map: HashMap<String, Value> = HashMap::new();
    match msg {
        ServerMessage::TimeSync(time_sync) => add_to_res(&mut map, "TimeSync", time_sync)?,
        ServerMessage::StateSync(state_sync) => add_to_res(&mut map, "StateSync", state_sync)?,
    };
    let json = serde_json::to_string(&map)?;
    Ok(json)
}
#[inline]
fn add_to_res<T: Serialize>(
    map: &mut HashMap<String, Value>,
    key: &str,
    value: T,
) -> Result<(), serde_json::Error> {
    let value = serde_json::to_value(value)?;
    map.insert(String::from("type"), Value::String(key.to_string()));
    map.insert(String::from("data"), value);
    Ok(())
}
