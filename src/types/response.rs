use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct WeChatResponse<T> {
    pub errcode: i32,
    pub errmsg: String,
    #[serde(flatten)]
    pub data: Option<T>,
}

impl<T> WeChatResponse<T> {
    pub fn is_success(&self) -> bool {
        self.errcode == 0
    }
}
