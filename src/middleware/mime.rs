use boluo::BoxError;
use boluo::headers::{ContentType, HeaderMapExt};
use boluo::response::{IntoResponse, Response};
use boluo::service::Service;
use mime::Mime;

pub async fn mime_with_charset<T, R, S>(
    charset: &T,
    request: R,
    service: &S,
) -> Result<Response, BoxError>
where
    T: ?Sized + AsRef<str>,
    S: ?Sized + Service<R>,
    S::Response: IntoResponse,
    S::Error: Into<BoxError>,
{
    let mut response = service.call(request).await.into_response()?;
    if let Some(mime) = response
        .headers()
        .typed_get::<ContentType>()
        .map(Mime::from)
    {
        if mime.type_() == mime::TEXT {
            let new_mime = format!("{}; charset={}", mime, charset.as_ref());
            if let Ok(new_content_type) = new_mime.parse::<ContentType>() {
                response.headers_mut().typed_insert(new_content_type);
            }
        }
    }
    Ok(response)
}
