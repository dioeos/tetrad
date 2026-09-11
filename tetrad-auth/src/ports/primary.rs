#[async_trait::async_trait]
pub trait AuthUseCases: Send + Sync {
    async fn login(&self);
    async fn cancel_login(&self);
    async fn restore_session(&self);
    async fn logout(&self);
    async fn status(&self);
}
