#![deny(unsafe_op_in_unsafe_fn)]
use jiff::Timestamp;
use std::cell::OnceCell;

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, NSMenu, NSMenuDelegate,
    NSSquareStatusItemLength, NSStatusBar, NSStatusItem,
};
use objc2_foundation::{
    MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSString, ns_string,
};

#[derive(Debug, Default)]
struct AppDelegateIvars {
    status_item: OnceCell<Retained<NSStatusItem>>,
}

define_class!(
    // SAFETY:
    // - The superclass NSObject does not have any subclassing requirements.
    // - `Delegate` does not implement `Drop`.
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = AppDelegateIvars]
    struct Delegate;

    // SAFETY: `NSObjectProtocol` has no safety requirements.
    unsafe impl NSObjectProtocol for Delegate {}

    // SAFETY: `NSApplicationDelegate` has no safety requirements.
    unsafe impl NSApplicationDelegate for Delegate {
        // SAFETY: The signature is correct.
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn did_finish_launching(&self, notification: &NSNotification) {
            let mtm = self.mtm();

            let app = unsafe { notification.object() }
                .unwrap()
                .downcast::<NSApplication>()
                .unwrap();
            // let app = NSApplication::sharedApplication(mtm); // this works too

            // Initialize item on the status bar
            let status_bar = unsafe { NSStatusBar::systemStatusBar() };
            let status_item = unsafe { status_bar.statusItemWithLength(NSSquareStatusItemLength) };

            // Give status item a button
            let status_button = unsafe { status_item.button(mtm).unwrap() };
            unsafe { status_button.setTitle(ns_string!("🪿")) }

            // Setup a menu
            let menu = NSMenu::new(mtm);
            unsafe {
                menu.addItemWithTitle_action_keyEquivalent(
                    ns_string!("Status"),
                    None,
                    ns_string!(""),
                );
                menu.addItemWithTitle_action_keyEquivalent(
                    ns_string!("Send a Goose"),
                    Some(sel!(order_goose)),
                    ns_string!(""),
                );
                menu.addItemWithTitle_action_keyEquivalent(
                    ns_string!("Cancel Goose"),
                    Some(sel!(cancel_goose)),
                    ns_string!(""),
                );
                menu.addItemWithTitle_action_keyEquivalent(
                    ns_string!("Quit"),
                    Some(sel!(terminate:)),
                    ns_string!("q"),
                );
            }
            let proto: &ProtocolObject<dyn NSMenuDelegate> = ProtocolObject::from_ref(self);
            unsafe { menu.setDelegate(Some(proto))}

            // Give our menu to status item
            unsafe { status_item.setMenu(Some(&menu)) }

            // Store the status item in the delegate vars.
            self.ivars().status_item.set(status_item).unwrap();

            app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

            // Activate the application.
            // Required when launching unbundled (as is done with Cargo).
            #[allow(deprecated)]
            app.activateIgnoringOtherApps(true);
        }
    }
    unsafe impl NSMenuDelegate for Delegate {
        #[unsafe(method(menuWillOpen:))]
        fn menu_will_open(&self, menu: &NSMenu) {
            let syst = Timestamp::now().to_string();
            unsafe { menu.itemAtIndex(0).unwrap().setTitle(&NSString::from_str(&syst)) }
        }
    }
    impl Delegate {
        #[unsafe(method(order_goose))]
        fn order_goose(&self) {
            println!("A goose is en route to your location!");
        }

        #[unsafe(method(cancel_goose))]
        fn cancel_goose(&self) {
            println!("Cancelling your goose :(");
        }
    }
);

impl Delegate {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(AppDelegateIvars::default());
        // SAFETY: The signature of `NSObject`'s `init` method is correct.
        unsafe { msg_send![super(this), init] }
    }
}

fn main() {
    let mtm = MainThreadMarker::new().unwrap();

    let app = NSApplication::sharedApplication(mtm);
    let delegate = Delegate::new(mtm);
    app.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));

    app.run();
}
