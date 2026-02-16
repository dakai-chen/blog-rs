use std::net::IpAddr;

use boluo::BoxError;
use boluo::data::Extension;
use boluo::extract::FromRequest;
use boluo::listener::ConnectInfo;
use boluo::request::Request;

#[derive(Debug, Clone)]
pub struct ClientIP(pub IpAddr);

impl FromRequest for ClientIP {
    type Error = BoxError;

    async fn from_request(request: &mut Request) -> Result<Self, Self::Error> {
        if let Some(ip) = request
            .headers()
            .get("x-forwarded-for")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<IpAddr>().ok())
        {
            return Ok(ClientIP(ip));
        }
        if let Some(ip) = request
            .headers()
            .get("x-real-ip")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<IpAddr>().ok())
        {
            return Ok(ClientIP(ip));
        }
        Ok(Extension::<ConnectInfo>::from_request(request)
            .await
            .map(|info| ClientIP(info.remote.ip()))?)
    }
}
