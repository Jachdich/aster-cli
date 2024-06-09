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
pub struct SyncData {
    pub user_uuid: i64,
    pub uname: String,
    pub pfp: String,
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
    #[serde(rename = "register")]         RegisterResponse       { status: Status, uuid: Option<i64> },
    #[serde(rename = "login")]            LoginResponse          { status: Status, uuid: Option<i64> },
    #[serde(rename = "get_metadata")]     GetMetadataResponse    { status: Status, data: Option<Vec<User>> },
    #[serde(rename = "sync_get_servers")] SyncGetServersResponse { status: Status, servers: Option<Vec<SyncServer>> },
    #[serde(rename = "online")]           OnlineResponse         { status: Status, data: Option<Vec<i64>> },
    #[serde(rename = "history")]          HistoryResponse        { status: Status, data: Option<Vec<Message>> },
    #[serde(rename = "get_user")]         GetUserResponse        { status: Status, data: Option<User> },
    #[serde(rename = "get_icon")]         GetIconResponse        { status: Status, data: Option<String> },
    #[serde(rename = "get_name")]         GetNameResponse        { status: Status, data: Option<String> },
    #[serde(rename = "list_channels")]    ListChannelsResponse   { status: Status, data: Option<Vec<Channel>> },
    #[serde(rename = "get_emoji")]        GetEmojiResponse       { status: Status, data: Option<Emoji> },
    #[serde(rename = "list_emoji")]       ListEmojiResponse      { status: Status, data: Option<Vec<(String, i64)>> },
    #[serde(rename = "sync_get")]         SyncGetResponse        { status: Status, #[serde(flatten)] data: Option<SyncData> },
    #[serde(rename = "content")]          ContentResponse        { status: Status, #[serde(flatten)] message: Message },
    #[serde(rename = "API_version")]      APIVersion             { status: Status, version: [u8; 3] },
    #[serde(rename = "send")]             SendResponse           { status: Status },
}
