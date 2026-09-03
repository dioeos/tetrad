use torii::UserId;
use uuid::Uuid;

pub(crate) struct Profile {
    pub(crate) torii_user_id: UserId,
    pub(crate) external_id: Uuid,
}

pub(crate) struct NewProfile {
    pub(crate) torii_user_id: UserId,
    pub(crate) external_id: Uuid
}
