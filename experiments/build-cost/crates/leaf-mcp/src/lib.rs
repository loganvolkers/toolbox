use rmcp::model::{ServerInfo, Implementation};
pub fn info() -> String {
    let mut i = ServerInfo::default();
    i.server_info = Implementation::from_build_env();
    serde_json::to_string(&i).unwrap()
}
