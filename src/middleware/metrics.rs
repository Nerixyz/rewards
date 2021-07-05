use actix_utils::future::{ok, Ready};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::error::Error;
use futures::future::Either;
use futures::ready;
use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

pub struct Metrics(Rc<Inner>);

struct Inner {
    ignored: HashSet<String>,
}

impl Metrics {
    pub fn new() -> Self {
        Self(Rc::new(Inner {
            ignored: HashSet::new(),
        }))
    }

    pub fn ignore<T: Into<String>>(mut self, path: T) -> Self {
        Rc::get_mut(&mut self.0)
            .unwrap()
            .ignored
            .insert(path.into());
        self
    }

    pub fn register_metrics() {
        metrics::register_histogram!(
            "http_request_duration",
            metrics::Unit::Milliseconds,
            "Request processing duration"
        );
        metrics::register_counter!("http_requests", "Total requests");
    }
}

impl<S> Transform<S, ServiceRequest> for Metrics
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Transform = MetricsMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(MetricsMiddleware {
            service,
            inner: self.0.clone(),
        })
    }
}

pub struct MetricsMiddleware<S> {
    inner: Rc<Inner>,
    service: S,
}

impl<S> Service<ServiceRequest> for MetricsMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = Either<S::Future, MetricsResponse<S>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if self.inner.ignored.contains(req.path()) {
            Either::Left(self.service.call(req))
        } else {
            let now = Instant::now();

            Either::Right(MetricsResponse {
                fut: self.service.call(req),
                start: now,
            })
        }
    }
}

#[pin_project::pin_project]
pub struct MetricsResponse<S>
where
    S: Service<ServiceRequest>,
{
    #[pin]
    fut: S::Future,
    start: Instant,
}

impl<S> Future for MetricsResponse<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
{
    type Output = Result<ServiceResponse, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let res: ServiceResponse = match ready!(this.fut.poll(cx)) {
            Ok(res) => res,
            Err(e) => return Poll::Ready(Err(e)),
        };

        let duration = Instant::now()
            .checked_duration_since(*this.start)
            .unwrap_or_else(|| Duration::from_secs(0));
        metrics::histogram!(
            "http_request_duration",
            duration.as_millis() as f64,
            "path" => res.request().path().to_owned(),
            "status" => res.status().as_str().to_owned(),
            "method" => res.request().method().as_str().to_owned()
        );
        metrics::increment_counter!("http_requests",
            "path" => res.request().path().to_owned(),
            "status" => res.status().as_str().to_owned(),
            "method" => res.request().method().as_str().to_owned()
        );

        Poll::Ready(Ok(res))
    }
}
