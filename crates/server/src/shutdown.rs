//! The one future every way of stopping the dashboard resolves.
//!
//! Two things end a dashboard: the idle deadline nobody asked to extend, and a
//! `POST` to the shutdown route. They must not be two shutdown paths. A second
//! path is a second set of ordering bugs, and only one of the two is exercised
//! by the tests that already exist — so the route resolves the future the idle
//! exit was already resolving, and everything downstream of a stop is written
//! once.
//!
//! A [`watch`] channel rather than a `Notify`, and the reason is not
//! preference. `Notify::notify_one` hands its permit to exactly one waiter, and
//! there is more than one: the graceful-shutdown future, plus every open
//! server-sent-events stream, which has to end for the shutdown to be able to
//! drain at all. `notify_waiters` has the opposite failure — a stop asked for
//! before anything is waiting is lost. A watched `bool` is seen by every
//! waiter, including one that starts waiting afterwards.

use tokio::sync::watch;

/// A stop that has been asked for, or has not yet.
pub struct Stop(watch::Sender<bool>);

impl Default for Stop {
    fn default() -> Self {
        Self(watch::channel(false).0)
    }
}

impl Stop {
    /// Ask the server to begin its graceful shutdown.
    ///
    /// Idempotent by construction: this sets a `bool` that is already `true` the
    /// second time, so a shutdown arriving alongside the idle exit — or two
    /// shutdown requests racing — is a no-op rather than a panic.
    pub fn request(&self) {
        // `send_replace` rather than `send`: `send` refuses when every receiver
        // has been dropped and leaves the value alone, and a server with no page
        // subscribed and nothing awaiting the shutdown yet is exactly that.
        self.0.send_replace(true);
    }

    /// Resolve once a stop has been asked for, including one asked for already.
    pub async fn requested(&self) {
        // `wait_for` checks the current value before waiting, so a request that
        // landed before this was awaited is not missed.
        let _ = self.0.subscribe().wait_for(|requested| *requested).await;
    }

    /// A watch on the same flag, for a caller that has to end a stream rather
    /// than await a future.
    pub fn receiver(&self) -> watch::Receiver<bool> {
        self.0.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    const PROMPTLY: Duration = Duration::from_secs(1);

    #[tokio::test]
    async fn a_stop_asked_for_before_anything_waits_is_not_lost() {
        let stop = Stop::default();

        stop.request();

        tokio::time::timeout(PROMPTLY, stop.requested())
            .await
            .expect("a stop asked for before the wait began never arrived");
    }

    /// The idle exit and the route converge here, so a shutdown landing on a
    /// server that is already shutting down must be an ordinary no-op.
    #[tokio::test]
    async fn asking_twice_is_a_no_op() {
        let stop = Stop::default();

        stop.request();
        stop.request();

        tokio::time::timeout(PROMPTLY, stop.requested())
            .await
            .expect("a second request broke the first");
    }

    /// Every open event stream waits on this too, and all of them have to end or
    /// the graceful shutdown never drains.
    #[tokio::test]
    async fn every_waiter_is_released_not_just_the_first() {
        let stop = Stop::default();

        let waiters = async {
            tokio::join!(stop.requested(), stop.requested(), stop.requested());
        };
        stop.request();

        tokio::time::timeout(PROMPTLY, waiters)
            .await
            .expect("a waiter was left behind");
    }

    #[tokio::test]
    async fn nothing_resolves_until_a_stop_is_asked_for() {
        let stop = Stop::default();

        assert!(
            tokio::time::timeout(Duration::from_millis(100), stop.requested())
                .await
                .is_err(),
            "the server stopped without being asked"
        );
    }
}
