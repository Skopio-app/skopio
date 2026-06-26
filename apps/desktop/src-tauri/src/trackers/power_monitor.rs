#![cfg(target_os = "macos")]

use block2::StackBlock;
use chrono::{DateTime, Utc};
use objc2::{class, msg_send, rc::Retained, runtime::AnyObject};
use objc2_foundation::{NSAutoreleasePool, NSNotification, NSString};
use tokio::sync::broadcast;
use tracing::error;

#[derive(Debug, Clone)]
pub enum PowerEvent {
    WillSleep { at: DateTime<Utc> },
    DidWake { at: DateTime<Utc> },
}

#[derive(Clone)]
pub struct PowerMonitor {
    tx: broadcast::Sender<PowerEvent>,
}

impl PowerMonitor {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(16);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<PowerEvent> {
        self.tx.subscribe()
    }

    pub fn start(&self) {
        let tx = self.tx.clone();

        std::thread::spawn(move || {
            if let Err(err) = start_nsworkspace_power_observers(tx) {
                error!("Failed to start power monitor: {err}");
            }
        });
    }
}

impl Default for PowerMonitor {
    fn default() -> Self {
        Self::new()
    }
}

fn start_nsworkspace_power_observers(tx: broadcast::Sender<PowerEvent>) -> Result<(), String> {
    unsafe {
        let _pool = NSAutoreleasePool::new();

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
        let did_wake_name: Retained<NSString> =
            NSString::from_str("NSWorkspaceDidWakeNotification");

        let _sleep_observer: *mut AnyObject = msg_send![
            center,
            addObserverForName: &*will_sleep_name,
            object: std::ptr::null_mut::<AnyObject>(),
            queue: std::ptr::null_mut::<AnyObject>(),
            usingBlock: &*sleep_block
        ];

        let _wake_observer: *mut AnyObject = msg_send![
            center,
            addObserverForName: &*did_wake_name,
            object: std::ptr::null_mut::<AnyObject>(),
            queue: std::ptr::null_mut::<AnyObject>(),
            usingBlock: &*wake_block
        ];

        let run_loop: *mut AnyObject = msg_send![class!(NSRunLoop), currentRunLoop];
        let distant_future: *mut AnyObject = msg_send![class!(NSDate), distantFuture];

        loop {
            let _: () = msg_send![run_loop, runUntilDate: distant_future];
        }
    }
}
