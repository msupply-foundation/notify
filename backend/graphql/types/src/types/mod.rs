pub mod recipient;
pub use self::recipient::*;
pub mod recipient_list;
pub use self::recipient_list::*;
pub mod user_account;
pub use self::user_account::*;
pub mod log;
pub use self::log::*;

use async_graphql::*;
pub struct DeleteResponse(pub String);
#[Object]
impl DeleteResponse {
    pub async fn id(&self) -> &str {
        &self.0
    }
}

pub struct IdResponse(pub String);
#[Object]
impl IdResponse {
    pub async fn id(&self) -> &str {
        &self.0
    }
}
