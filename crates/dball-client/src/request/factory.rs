use std::sync::Arc;

use super::provider::{Provider, QpsLimitedExecutor};

/// Factory trait for creating API requests for a specific provider
pub trait ApiFactory: Provider {
    /// Create a request and return it with the associated executor
    fn create_request<R>(&self, request: R) -> FactoryRequest<R>
    where
        R: super::provider::ProviderRequest,
    {
        FactoryRequest {
            request,
            executor: self.get_executor(),
        }
    }
}

/// A request created by a factory, containing both the request and its executor
pub struct FactoryRequest<R>
where
    R: super::provider::ProviderRequest,
{
    request: R,
    executor: Arc<QpsLimitedExecutor>,
}

impl<R> FactoryRequest<R>
where
    R: super::provider::ProviderRequest,
{
    /// Execute the request through the provider's QPS-limited executor
    pub async fn execute(self, common: &super::ApiCommon) -> anyhow::Result<R::Response> {
        self.executor.execute(self.request, common).await
    }
}

/// Macro to implement `ApiFactory` for a Provider type
#[macro_export]
macro_rules! impl_api_factory {
    ($type:ty) => {
        impl $crate::request::factory::ApiFactory for $type {}
    };
}

pub use impl_api_factory;
