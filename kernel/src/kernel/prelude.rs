use spin::{lazy::Lazy, mutex::Mutex};

use crate::kernel::EventManager;

pub static KERNEL_EVENT_MANAGER: Lazy<Mutex<EventManager>> = Lazy::new(|| {
    Mutex::new(EventManager::new())
});

#[macro_export]
macro_rules! kprintln {
    ($text:expr, $($args:tt)*) => {
        let formatted = $crate::alloc::format!($text, $($args)*);
        $crate::kernel::prelude::KERNEL_EVENT_MANAGER.lock()
            .new_event($crate::kernel::EventType::PrintLine(formatted));
    };

    ($text:expr) => {
        use alloc::string::ToString;

        $crate::kernel::prelude::KERNEL_EVENT_MANAGER.lock()
            .new_event($crate::kernel::EventType::PrintLine($text.to_string()));
    };
}