#![cfg(target_os = "macos")]

use block2::StackBlock;
use chrono::{DateTime, Utc};
use objc2::{
    class, msg_send,
    rc::{Retained, autoreleasepool},
    runtime::AnyObject,
};
use objc2_foundation::{NSNotification, NSString};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::broadcast;
use tracing::{error, info};

use crate::trackers::TrackerLifecycle;

#[derive(Debug, Clone)]
pub enum PowerEvent {
    WillSleep { at: DateTime<Utc> },
    DidWake { at: DateTime<Utc> },
}

#[derive(Clone)]
pub struct PowerMonitor {
    tx: broadcast::Sender<PowerEvent>,
    shutdown: Arc<AtomicBool>,
}

impl PowerMonitor {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(16);
        Self {
            tx,
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<PowerEvent> {
        self.tx.subscribe()
    }

    pub fn start(&self) {
        self.shutdown.store(false, Ordering::Relaxed);
        let tx = self.tx.clone();
        let shutdown = Arc::clone(&self.shutdown);

        match std::thread::Builder::new()
            .name("skopio-power-monitor".into())
            .spawn(move || {
                if let Err(error) = run_power_monitor(tx, shutdown) {
                    error!("Failed to start power monitor: {error}");
                }
            }) {
            Ok(_handle) => {
                // Dropping JoinHandle detaches the thread, which is fine here
            }
            Err(error) => {
                error!("Failed to spawn monitor thread: {error}");
            }
        }
    }

    pub fn stop_tracking(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }
}

#[async_trait::async_trait]
impl TrackerLifecycle for PowerMonitor {
    type StartArgs = ();

    fn start_tracking(self: Arc<Self>, (): Self::StartArgs) {
        self.start();
    }

    async fn shutdown(&self) {
        self.stop_tracking();
    }
}

impl Default for PowerMonitor {
    fn default() -> Self {
        Self::new()
    }
}

fn run_power_monitor(
    tx: broadcast::Sender<PowerEvent>,
    shutdown: Arc<AtomicBool>,
) -> Result<(), String> {
    let run_loop = autoreleasepool(|_| unsafe {
        register_power_observers(tx)?;
        let run_loop: *mut AnyObject = msg_send![class!(NSRunLoop), currentRunLoop];

        if run_loop.is_null() {
            return Err("NSRunLoop.currentLoop returned null".into());
        }

        Ok::<*mut AnyObject, String>(run_loop)
    })?;

    while !shutdown.load(Ordering::Relaxed) {
        autoreleasepool(|_| unsafe {
            let until: *mut AnyObject =
                msg_send![class!(NSDate), dateWithTimeIntervalSinceNow: 1.0f64];
            let _: () = msg_send![run_loop, runUntilDate: until];
        })
    }

    info!("Power monitor stopped");
    Ok(())
}

unsafe fn register_power_observers(tx: broadcast::Sender<PowerEvent>) -> Result<(), String> {
    let workspace: *mut AnyObject = msg_send![class!(NSWorkspace), sharedWorkspace];
    if workspace.is_null() {
        return Err("NSWorkspace.sharedWorkspace returned null".into());
    }

    let center: *mut AnyObject = msg_send![workspace, notificationCenter];
    if center.is_null() {
        return Err("NSWorkspace notificationCenter returned null".into());
    }

    let sleep_tx = tx.clone();
    let sleep_block = StackBlock::new(move |_notification: *mut NSNotification| {
        let _ = sleep_tx.send(PowerEvent::WillSleep { at: Utc::now() });
    })
    .copy();

    let wake_tx = tx;
    let wake_block = StackBlock::new(move |_notification: *mut NSNotification| {
        let _ = wake_tx.send(PowerEvent::DidWake { at: Utc::now() });
    })
    .copy();

    let will_sleep_name: Retained<NSString> =
        NSString::from_str("NSWorkspaceWillSleepNotification");
    let did_wake_name: Retained<NSString> = NSString::from_str("NSWorkspaceDidWakeNotification");

    let sleep_observer: *mut AnyObject = msg_send![
        center,
        addObserverForName: &*will_sleep_name,
        object: std::ptr::null_mut::<AnyObject>(),
        queue: std::ptr::null_mut::<AnyObject>(),
        usingBlock: &*sleep_block
    ];

    let wake_observer: *mut AnyObject = msg_send![
        center,
        addObserverForName: &*did_wake_name,
        object: std::ptr::null_mut::<AnyObject>(),
        queue: std::ptr::null_mut::<AnyObject>(),
        usingBlock: &*wake_block
    ];

    if sleep_observer.is_null() || wake_observer.is_null() {
        return Err("Failed to register NSWorkspace power observers".into());
    }

    Ok(())
}
