//! When a dashboard nobody needs gives up.
//!
//! The idle condition takes two inputs, and both are required:
//!
//! ```text
//!   exit when   no page has been subscribed for the window
//!         AND   no activity has been registered for the window
//! ```
//!
//! Keying it on the subscriber count alone is wrong, and the review loop's own
//! asynchrony is what breaks it: the reviewer skims a note, closes the tab, and
//! walks away, while the session keeps working towards the verdict the dashboard
//! exists to collect. An idle timer that assumes the reviewer is present
//! whenever the dashboard matters has the premise backwards.
//!
//! The second input needs no new mechanism. The turn-end hook has to probe the
//! identity route anyway to decide whether to start a dashboard, so **the probe
//! is the heartbeat**: one request does both jobs.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::watch::Hub;

/// How often the idle condition is rechecked, at most. A thirty-minute window
/// does not deserve a busy loop; a test's short one needs the check to keep up
/// with it, which is what dividing the window covers.
const CHECK_INTERVAL: Duration = Duration::from_secs(30);

/// When the dashboard was last asked for by something outside it.
pub struct Activity(Mutex<Instant>);

impl Default for Activity {
    fn default() -> Self {
        Self(Mutex::new(Instant::now()))
    }
}

impl Activity {
    /// Push the idle deadline out: something outside the dashboard just wanted
    /// it.
    pub fn register(&self) {
        *self.0.lock().expect("activity lock") = Instant::now();
    }

    fn elapsed(&self) -> Duration {
        self.0.lock().expect("activity lock").elapsed()
    }
}

/// Resolve once nothing has needed the dashboard for `window`: no page
/// subscribed, and no activity registered.
pub async fn unused(hub: &Hub, activity: &Activity, window: Duration) {
    let mut ticker = tokio::time::interval(check_interval(window));
    ticker.tick().await; // The first tick completes immediately.

    loop {
        ticker.tick().await;

        if hub.subscribers() == 0 && activity.elapsed() >= window {
            return;
        }
    }
}

fn check_interval(window: Duration) -> Duration {
    (window / 4)
        .min(CHECK_INTERVAL)
        .max(Duration::from_millis(10))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    use crate::watch::Target;

    const WINDOW: Duration = Duration::from_millis(200);

    fn target() -> Target {
        Target {
            watched_dirs: Vec::new(),
            artifacts: vec![std::env::temp_dir().join("openspec-doc-idle-test")],
            review_state: Vec::new(),
        }
    }

    #[tokio::test]
    async fn an_unused_dashboard_gives_up() {
        let hub = Hub::default();
        let activity = Activity::default();

        assert!(
            tokio::time::timeout(WINDOW * 20, unused(&hub, &activity, WINDOW))
                .await
                .is_ok(),
            "an idle dashboard never gave up"
        );
    }

    /// A tab left open is a reviewer who may come back to it.
    #[tokio::test]
    async fn a_subscribed_page_holds_the_dashboard_open() {
        let hub = Hub::default();
        let activity = Activity::default();
        let _page = hub.subscribe(&target());

        assert!(
            tokio::time::timeout(WINDOW * 5, unused(&hub, &activity, WINDOW))
                .await
                .is_err(),
            "a subscribed page did not hold the dashboard open"
        );
    }

    /// The case a subscriber-only check gets wrong: every tab closed, and a
    /// session still working towards the verdict the dashboard collects.
    #[tokio::test]
    async fn registered_activity_holds_the_dashboard_open_with_no_page_subscribed() {
        let hub = Hub::default();
        let activity = Activity::default();

        let held = tokio::time::timeout(WINDOW * 5, async {
            let heartbeat = async {
                loop {
                    tokio::time::sleep(WINDOW / 4).await;
                    activity.register();
                }
            };
            tokio::select! {
                () = unused(&hub, &activity, WINDOW) => (),
                () = heartbeat => (),
            }
        })
        .await;

        assert!(held.is_err(), "a heartbeat did not hold the dashboard open");
    }
}
