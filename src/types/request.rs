use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Request<T> {
    pub appid: String,
    #[serde(flatten)]
    pub body: T,
}
