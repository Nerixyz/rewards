use crate::{log_err, services::supinic::update_activity};
use actix::{Actor, AsyncContext, Context, ContextFutureSpawner, WrapFuture};
use std::time::Duration;

pub struct SupinicActor;

impl Actor for SupinicActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_secs(15 * 60), |this, ctx| {
            async move {
                log_err!(
                    update_activity().await,
                    "Could not update supi activity"
                );
            }
            .into_actor(this)
            .spawn(ctx)
        });
    }
}
