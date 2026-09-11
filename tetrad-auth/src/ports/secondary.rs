#[async_trait::async_trait]
pub trait IdentityProvider: Send + Sync {
    async fn prepare_login(&self);
    async fn exchange_code(&self);
}
