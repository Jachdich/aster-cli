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
}

#[derive(Serialize, Deserialize)]
pub struct SyncServer {
    pub user_uuid: i64,
    pub server_uuid: i64,
    pub ip: String,
    pub port: i32,
    pub pfp: Option<String>,
    pub name: Option<String>,
    pub idx: i32,
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
pub struct Message {
    pub uuid: i64,
    pub content: String,
    pub author_uuid: i64,
    pub channel_uuid: i64,
    pub date: i32,
}

#[derive(Serialize)]
#[serde(tag = "command")]
#[rustfmt::skip]
pub enum Request {
    #[serde(rename = "register")]         RegisterRequest { passwd: String, uname: String },
    #[serde(rename = "login")]            LoginRequest { passwd: String, uname: Option<String>, uuid: Option<i64> },
    #[serde(rename = "ping")]             PingRequest,
    #[serde(rename = "nick")]             NickRequest { nick: String },
    #[serde(rename = "online")]           OnlineRequest,
    #[serde(rename = "send")]             SendRequest { content: String, channel: i64 },
    #[serde(rename = "get_metadata")]     GetMetadataRequest,
    #[serde(rename = "get_name")]         GetNameRequest,
    #[serde(rename = "get_icon")]         GetIconRequest,
    #[serde(rename = "list_emoji")]       ListEmojiRequest,
    #[serde(rename = "get_emoji")]        GetEmojiRequest { uuid: i64 },
    #[serde(rename = "list_channels")]    ListChannelsRequest,
    #[serde(rename = "history")]          HistoryRequest { num: u32, channel: i64, before_message: Option<i64> },
    #[serde(rename = "pfp")]              PfpRequest { data: String },
    #[serde(rename = "sync_set")]         SyncSetRequest { uname: String, pfp: String },
    #[serde(rename = "sync_get")]         SyncGetRequest,
    #[serde(rename = "sync_set_servers")] SyncSetServersRequest { severs: Vec<SyncServer> },
    #[serde(rename = "sync_get_servers")] SyncGetServersRequest,
    #[serde(rename = "leave")]            LeaveRequest,
    #[serde(rename = "get_user")]         GetUserRequest { uuid: i64 },
}

#[derive(Deserialize)]
#[serde(tag = "command")]
#[rustfmt::skip]
pub enum Response {  
    #[serde(rename = "register")]         RegisterResponse       { uuid: i64 },
    #[serde(rename = "login")]            LoginResponse          { uuid: i64 },
    #[serde(rename = "get_metadata")]     GetMetadataResponse    { data: Vec<User> },
    #[serde(rename = "sync_get_servers")] SyncGetServersResponse { servers: Vec<SyncServer> },
    #[serde(rename = "online")]           OnlineResponse         { data: Vec<i64> },
    #[serde(rename = "history")]          HistoryResponse        { data: Vec<Message> },
    #[serde(rename = "get_user")]         GetUserResponse        { data: User },
    #[serde(rename = "get_icon")]         GetIconResponse        { data: String },
    #[serde(rename = "get_name")]         GetNameResponse        { data: String },
    #[serde(rename = "list_channels")]    ListChannelsResponse   { data: Vec<Channel> },
    #[serde(rename = "get_emoji")]        GetEmojiResponse       { data: Emoji },
    #[serde(rename = "list_emoji")]       ListEmojiResponse      { data: Vec<(String, i64)> },
    #[serde(rename = "sync_get")]         SyncGetResponse        { user_uuid: i64, uname: String, pfp: String },
    #[serde(rename = "content")]          ContentResponse(Message), // TODO does this (de)serialise correctly?
    #[serde(rename = "API_version")]      APIVersion { version: [u8; 3] },
    #[serde(rename = "send")] SendResponse,
}
