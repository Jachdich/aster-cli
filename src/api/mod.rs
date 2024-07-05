#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize_repr, Deserialize_repr, Clone, Copy, Debug, PartialEq)]
#[repr(u16)]
pub enum Status {
    Ok = 200,
    BadRequest = 400,
    InternalError = 500,
    Unauthorised = 401,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    Conflict = 409,
}

impl Status {
    fn name(&self) -> String {
        match self {
            Status::Ok => "Ok",
            Status::BadRequest => "Bad Request",
            Status::InternalError => "Internal Error",
            Status::Unauthorised => "Unauthorised",
            Status::Forbidden => "Forbidden",
            Status::NotFound => "Not Found",
            Status::MethodNotAllowed => "Method Not Allowed",
            Status::Conflict => "Conflict",
        }
        .to_owned()
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let name = self.name();
        write!(f, "{}", name)
    }
}

#[derive(Serialize, Deserialize)]
pub struct SyncServer {
    pub uuid: Option<i64>,
    pub uname: String,
    pub ip: String,
    pub port: i32,
    pub pfp: Option<String>,
    pub name: Option<String>,
    pub idx: i32,
}

#[derive(Deserialize)]
pub struct SyncData {
    pub user_uuid: i64,
    pub uname: String,
    pub pfp: String,
}

#[derive(Deserialize, Clone)]
pub struct User {
    pub uuid: i64,
    pub name: String,
    pub pfp: String,
    pub group_uuid: i64,
}

#[derive(Deserialize)]
pub struct Emoji {
    pub uuid: i64,
    pub name: String,
    pub data: String,
}

#[derive(Deserialize)]
pub struct Channel {
    pub uuid: i64,
    pub name: String,
}

#[derive(Deserialize, Clone)]
pub struct Message {
    pub uuid: i64,
    pub content: String,
    pub author_uuid: i64,
    pub channel_uuid: i64,
    pub date: i32,
    #[serde(default)]
    pub edited: bool,
}

#[derive(Serialize)]
#[serde(tag = "command")]
#[rustfmt::skip]
pub enum Request {
    #[serde(rename = "register")]         Register { passwd: String, uname: String },
    #[serde(rename = "login")]            Login { passwd: String, uname: Option<String>, uuid: Option<i64> },
    #[serde(rename = "ping")]             Ping,
    #[serde(rename = "nick")]             Nick { nick: String },
    #[serde(rename = "online")]           Online,
    #[serde(rename = "send")]             Send { content: String, channel: i64 },
    #[serde(rename = "get_metadata")]     GetMetadata,
    #[serde(rename = "get_name")]         GetName,
    #[serde(rename = "get_icon")]         GetIcon,
    #[serde(rename = "list_emoji")]       ListEmoji,
    #[serde(rename = "get_emoji")]        GetEmoji { uuid: i64 },
    #[serde(rename = "list_channels")]    ListChannels,
    #[serde(rename = "history")]          History { num: u32, channel: i64, before_message: Option<i64> },
    #[serde(rename = "pfp")]              Pfp { data: String },
    #[serde(rename = "sync_set")]         SyncSet { uname: String, pfp: String },
    #[serde(rename = "sync_get")]         SyncGet,
    #[serde(rename = "sync_set_servers")] SyncSetServers { severs: Vec<SyncServer> },
    #[serde(rename = "sync_get_servers")] SyncGetServers,
    #[serde(rename = "leave")]            Leave,
    #[serde(rename = "get_user")]         GetUser { uuid: i64 },
    #[serde(rename = "edit")]             Edit { message: i64, new_content: String },
    #[serde(rename = "delete")]           Delete { message: i64 },
}

#[derive(Deserialize)]
#[serde(tag = "command")]
#[rustfmt::skip]
pub enum Response {  
    #[serde(rename = "register")]         Register       { status: Status, uuid: Option<i64> },
    #[serde(rename = "login")]            Login          { status: Status, uuid: Option<i64> },
    #[serde(rename = "get_metadata")]     GetMetadata    { status: Status, data: Option<Vec<User>> },
    #[serde(rename = "sync_get_servers")] SyncGetServers { status: Status, servers: Option<Vec<SyncServer>> },
    #[serde(rename = "online")]           Online         { status: Status, data: Option<Vec<i64>> },
    #[serde(rename = "history")]          History        { status: Status, data: Option<Vec<Message>> },
    #[serde(rename = "get_user")]         GetUser        { status: Status, data: Option<User> },
    #[serde(rename = "get_icon")]         GetIcon        { status: Status, data: Option<String> },
    #[serde(rename = "get_name")]         GetName        { status: Status, data: Option<String> },
    #[serde(rename = "list_channels")]    ListChannels   { status: Status, data: Option<Vec<Channel>> },
    #[serde(rename = "get_emoji")]        GetEmoji       { status: Status, data: Option<Emoji> },
    #[serde(rename = "list_emoji")]       ListEmoji      { status: Status, data: Option<Vec<(String, i64)>> },
    #[serde(rename = "sync_get")]         SyncGet        { status: Status, #[serde(flatten)] data: Option<SyncData> },
    #[serde(rename = "content")]          Content        { status: Status, #[serde(flatten)] message: Message },
    #[serde(rename = "API_version")]      APIVersion     { status: Status, version: [u8; 3] },
    #[serde(rename = "send")]             Send           { status: Status, message: i64, },
    #[serde(rename = "edit")]             Edit           { status: Status },
    #[serde(rename = "delete")]           Delete         { status: Status },
    #[serde(rename = "message_edited")]   MessageEdited  { status: Status, message: i64, new_content: String },
    #[serde(rename = "message_deleted")]  MessageDeleted { status: Status, message: i64 },

}

impl Response {
    pub fn status(&self) -> Status {
        // helper to get the status of any event
        // wish there was a better way
        use Response::*;
        *match self {
            Register { status, .. } => status,
            Login { status, .. } => status,
            GetMetadata { status, .. } => status,
            SyncGetServers { status, .. } => status,
            Online { status, .. } => status,
            History { status, .. } => status,
            GetUser { status, .. } => status,
            GetIcon { status, .. } => status,
            GetName { status, .. } => status,
            ListChannels { status, .. } => status,
            GetEmoji { status, .. } => status,
            ListEmoji { status, .. } => status,
            SyncGet { status, .. } => status,
            Content { status, .. } => status,
            APIVersion { status, .. } => status,
            Send { status, .. } => status,
            MessageEdited { status, .. } => status,
            Edit { status, .. } => status,
            MessageDeleted { status, .. } => status,
            Delete { status, .. } => status,
        }
    }
    pub fn name(&self) -> &'static str {
        // likewise for the name
        use Response::*;
        match self {
            Register { .. } => "RegisterResponse",
            Login { .. } => "LoginResponse",
            GetMetadata { .. } => "GetMetadataResponse",
            SyncGetServers { .. } => "SyncGetServersResponse",
            Online { .. } => "OnlineResponse",
            History { .. } => "HistoryResponse",
            GetUser { .. } => "GetUserResponse",
            GetIcon { .. } => "GetIconResponse",
            GetName { .. } => "GetNameResponse",
            ListChannels { .. } => "ListChannelsResponse",
            GetEmoji { .. } => "GetEmojiResponse",
            ListEmoji { .. } => "ListEmojiResponse",
            SyncGet { .. } => "SyncGetResponse",
            Content { .. } => "ContentResponse",
            APIVersion { .. } => "APIVersion",
            Send { .. } => "SendResponse",
            MessageEdited { .. } => "MessageEditedResponse",
            Edit { .. } => "EditResponse",
            MessageDeleted { .. } => "MessageDeletedResponse",
            Delete { .. } => "DeleteResponse",
        }
    }
}
