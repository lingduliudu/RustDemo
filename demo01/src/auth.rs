use actix_web::{
    Error,
    dev::{ServiceRequest, ServiceResponse, forward_ready},
};
use futures_util::future::{LocalBoxFuture, Ready, ready};
pub struct TokenCheck;

impl<S, B> actix_web::dev::Transform<S, ServiceRequest> for TokenCheck
where
    S: actix_web::dev::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = TokenCheckMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TokenCheckMiddleware { service }))
    }
}

pub struct TokenCheckMiddleware<S> {
    service: S,
}

impl<S, B> actix_web::dev::Service<ServiceRequest> for TokenCheckMiddleware<S>
where
    S: actix_web::dev::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let token_header = req.headers().get("token").cloned();
        let fut = self.service.call(req);

        Box::pin(async move {
            if let Some(token) = token_header {
                if token == "1" {
                    // token 合法，继续处理请求
                    return fut.await;
                }
            }
            // token 缺失或不合法，直接返回 401
            Err(actix_web::error::ErrorImATeapot("Token invalid or missing"))
        })
    }
}
