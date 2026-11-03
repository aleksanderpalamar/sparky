#[cfg(test)]
mod app_tests;

use std::os::fd::OwnedFd;
use std::time::Duration;

use tokio::sync::mpsc::Receiver;
use tokio::time::{Instant, timeout_at};

use crate::domain::guard::{Guard, Input, Order};
use crate::ports::{Compositor, Logind};

pub const GRACE: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Seen(Input),
    CompositorGone,
}

pub async fn run(logind: &impl Logind, compositor: &impl Compositor, mut events: Receiver<Event>) {
    let (mut guard, orders) = Guard::start();
    let mut runtime = Runtime::default();
    runtime.perform(logind, compositor, orders).await;
    while let Some(input) = runtime.next(&mut events).await {
        let orders = guard.handle(input);
        runtime.perform(logind, compositor, orders).await;
    }
}

#[derive(Default)]
struct Runtime {
    held: Option<OwnedFd>,
    deadline: Option<Instant>,
}

impl Runtime {
    async fn next(&mut self, events: &mut Receiver<Event>) -> Option<Input> {
        let event = match self.deadline {
            Some(deadline) => match timeout_at(deadline, events.recv()).await {
                Ok(event) => event,
                Err(_) => {
                    self.deadline = None;
                    return Some(Input::Expired);
                }
            },
            None => events.recv().await,
        };
        match event? {
            Event::Seen(input) => Some(input),
            Event::CompositorGone => None,
        }
    }

    async fn perform(
        &mut self,
        logind: &impl Logind,
        compositor: &impl Compositor,
        orders: Vec<Order>,
    ) {
        for order in orders {
            if let Err(error) = self.apply(logind, compositor, order).await {
                eprintln!("sparky-lockd: {error}");
            }
        }
    }

    async fn apply(
        &mut self,
        logind: &impl Logind,
        compositor: &impl Compositor,
        order: Order,
    ) -> Result<(), String> {
        match order {
            Order::Lock => compositor.lock(),
            Order::Unlock => compositor.unlock(),
            Order::Hold => {
                self.held = Some(logind.inhibit().await?);
                Ok(())
            }
            Order::Release => {
                self.held = None;
                self.deadline = None;
                Ok(())
            }
            Order::Watch => {
                self.deadline = Some(Instant::now() + GRACE);
                Ok(())
            }
            Order::Hint(screen) => logind.hint(screen).await,
        }
    }
}
