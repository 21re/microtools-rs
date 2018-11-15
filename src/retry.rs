use super::{AsyncBusinessResult, BusinessResult};
use actix::fut;
use actix::prelude::*;
use futures::sync::oneshot;
use futures::Future;
use problem::Problem;
use std::time::Duration;

pub struct RetryActor<F, C, U> {
  delay: Duration,
  context: C,
  factory: F,
  sender: Option<oneshot::Sender<BusinessResult<U>>>,
}

pub struct Try(u16);

impl Message for Try {
  type Result = ();
}

impl<F, C, FU, U> Handler<Try> for RetryActor<F, C, U>
where
  F: Fn(&C) -> FU + 'static,
  FU: Future<Item = U, Error = Problem> + 'static,
  U: 'static + Send,
  C: 'static,
{
  type Result = ();

  fn handle(&mut self, msg: Try, ctx: &mut Self::Context) -> Self::Result {
    ctx.spawn(
      (self.factory)(&self.context)
        .into_actor(self)
        .then(move |result, actor, inner_ctx| {
          match result {
            Ok(value) => {
              if let Some(sender) = actor.sender.take() {
                match sender.send(Ok(value)) {
                  Ok(_) => (),
                  Err(_) => {
                    error!("Sender failed");
                  }
                }
              }
            }
            Err(_) if msg.0 > 1 => {
              inner_ctx.notify_later(Try(msg.0 - 1), actor.delay);
            }
            Err(problem) => {
              if let Some(sender) = actor.sender.take() {
                match sender.send(Err(problem)) {
                  Ok(_) => (),
                  Err(_) => {
                    error!("Sender failed");
                  }
                }
              }
            }
          };
          fut::ok(())
        }),
    );
    ()
  }
}

impl<F, C, FU, U> Actor for RetryActor<F, C, U>
where
  F: Fn(&C) -> FU + 'static,
  FU: Future<Item = U, Error = Problem> + 'static,
  U: 'static + Send,
  C: 'static,
{
  type Context = Context<Self>;
}

pub struct Retrier {
  tries: u16,
  delay: Duration,
}

impl Default for Retrier {
  fn default() -> Self {
    Retrier {
      tries: 5,
      delay: Duration::from_millis(500),
    }
  }
}

impl Retrier {
  pub fn new(tries: u16, delay: Duration) -> Retrier {
    Retrier { tries, delay }
  }

  pub fn retry<F, C, FU, U>(&self, context: C, factory: F) -> AsyncBusinessResult<U>
  where
    F: Fn(&C) -> FU + 'static,
    FU: Future<Item = U, Error = Problem> + 'static,
    U: 'static + Send,
    C: 'static,
  {
    let (sender, receiver) = oneshot::channel::<BusinessResult<U>>();
    let retrier = RetryActor {
      delay: self.delay,
      context,
      factory,
      sender: Some(sender),
    }.start();

    Box::new(
      retrier
        .send(Try(self.tries))
        .map_err(|err| Problem::internal_server_error().with_details(format!("{}", err)))
        .and_then(|_| {
          receiver
            .map_err(|err| Problem::internal_server_error().with_details(format!("{}", err)))
            .flatten()
        }),
    )
  }
}
