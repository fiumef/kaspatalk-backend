use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub(crate) struct GetMessagesQueryParameters {
    pub(crate)address_1: String,
    pub(crate)address_2: String,
}