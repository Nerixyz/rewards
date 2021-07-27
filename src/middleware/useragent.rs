use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::Error,
    http::{header::USER_AGENT, StatusCode},
    HttpResponse,
};
use futures::future::{ok, Either, Ready};
use std::{
    rc::Rc,
    task::{Context, Poll},
};

pub struct UserAgentGuard {
    banned: Rc<Vec<String>>,
}

impl UserAgentGuard {
    pub fn single(item: String) -> Self {
        Self::new(vec![item])
    }

    pub fn new(items: Vec<String>) -> Self {
        Self {
            banned: Rc::new(items),
        }
    }
}

impl<S> Transform<S, ServiceRequest> for UserAgentGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Transform = UserAgentMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(UserAgentMiddleware {
            service,
            banned: self.banned.clone(),
        })
    }
}

pub struct UserAgentMiddleware<S> {
    service: S,
    banned: Rc<Vec<String>>,
}

impl<S> Service<ServiceRequest> for UserAgentMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    #[allow(clippy::type_complexity)]
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let any_banned = req
            .headers()
            .get(USER_AGENT)
            .filter(|header| {
                header
                    .to_str()
                    .ok()
                    .filter(|ua| self.banned.iter().any(|banned| ua.contains(banned)))
                    .is_some()
            })
            .is_some();
        if any_banned {
            Either::Right(ok(
                req.into_response(HttpResponse::build(StatusCode::IM_A_TEAPOT).finish())
            ))
        } else {
            Either::Left(self.service.call(req))
        }
    }
}
