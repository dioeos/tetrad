use crate::instance::InstanceError;
use crate::error::HttpError;
use tracing::error;

impl From<InstanceError> for HttpError {
    fn from(error: InstanceError) -> Self {
        match error {
            InstanceError::NotFound => HttpError::not_found("instance not found"),
            InstanceError::Storage(source) => {
                error!(
                    error = ?source,
                    "instance storage operation failed"
                );
                HttpError::internal_server_error("failed to load instance")
            }
        }
    }
}
