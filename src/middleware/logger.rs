use std::time::Instant;

use boluo::extract::FromRequest;
use boluo::middleware::Middleware;
use boluo::response::{IntoResponse, Response};
use boluo::service::Service;
use boluo::{BoxError, request::Request};
use tracing::Instrument;

use crate::context::ip::ClientIP;

#[derive(Debug, Clone, Copy)]
pub struct Logger;

impl<S> Middleware<S> for Logger {
    type Service = LoggerService<S>;

    fn transform(self, service: S) -> Self::Service {
        LoggerService { inner: service }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LoggerService<S> {
    inner: S,
}

impl<S> Service<Request> for LoggerService<S>
where
    S: Service<Request>,
    S::Response: IntoResponse,
    S::Error: Into<BoxError>,
{
    type Response = Response;
    type Error = BoxError;

    async fn call(&self, request: Request) -> Result<Self::Response, Self::Error> {
        tracing::info!(
            "{:?} {} {}",
            request.version(),
            request.method(),
            request.uri()
        );
        let start = Instant::now(); // 计时开始
        let response = self.inner.call(request).await.into_response()?;
        let elapsed = start.elapsed(); // 计时结束
        tracing::info!("{}，执行耗时：{elapsed:.2?}", response.status());
        Ok(response)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LoggerSpan;

impl<S> Middleware<S> for LoggerSpan {
    type Service = LoggerSpanService<S>;

    fn transform(self, service: S) -> Self::Service {
        LoggerSpanService { inner: service }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LoggerSpanService<S> {
    inner: S,
}

impl<S> Service<Request> for LoggerSpanService<S>
where
    S: Service<Request>,
    S::Error: Into<BoxError>,
{
    type Response = S::Response;
    type Error = BoxError;

    async fn call(&self, mut request: Request) -> Result<Self::Response, Self::Error> {
        let ClientIP(ip) = ClientIP::from_request(&mut request).await?;
        let span = tracing::error_span!("HTTP", ip = %ip, request_id = tracing::field::Empty);
        span.in_scope(|| self.inner.call(request))
            .instrument(span)
            .await
            .map_err(Into::into)
    }
}
