use crate::acl::UserID;

pub struct Command {
    user: UserID,
    issue_time: chrono::DateTime<chrono::Utc>,
    addon_id: String,
    thing_uid: String,
    command_name: String,
    command_value: Option<serde_json::Value>
}
