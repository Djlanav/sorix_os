use alloc::format;
use spin::{lazy::Lazy, mutex::Mutex};

use crate::kernel::EventManager;

pub static KERNEL_EVENT_MANAGER: Lazy<Mutex<EventManager>> = Lazy::new(|| {
    Mutex::new(EventManager::new())
});

#[macro_export]
macro_rules! kprintln {
    ($text:expr, $($args:tt)*) => {
        use alloc::string::ToString;
        use alloc::format;
        use crate::kernel::prelude::KERNEL_EVENT_MANAGER;
        use crate::kernel::EventType;

        let formatted = format!($text, $($args)*);
        KERNEL_EVENT_MANAGER.lock().new_event(EventType::PrintLine(formatted.to_string()));
    };

    ($text:expr) => {
        use alloc::string::ToString;
        use crate::kernel::prelude::KERNEL_EVENT_MANAGER;
        use crate::kernel::EventType;

        KERNEL_EVENT_MANAGER.lock().new_event(EventType::PrintLine($text.to_string()));
    }
}