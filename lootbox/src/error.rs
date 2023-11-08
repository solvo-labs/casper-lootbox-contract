use casper_types::ApiError;

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum Error {
    FatalError = 0,
    AdminError = 1,
    NotApproved = 2,
    LootboxLimit = 3,
    ClaimNotFound = 4,
    MaxItemCount = 5,
    ItemNotFound = 6,
    RarityLevelNotFound = 7,
}

impl From<Error> for ApiError {
    fn from(error: Error) -> ApiError {
        ApiError::User(error as u16)
    }
}
