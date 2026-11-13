pub mod event;
pub mod instrument;
mod macros;
pub mod metadata;
pub mod span;
pub mod subscriber;

pub use korio_core::{dispatcher, format_key};

#[doc(inline)]
pub use self::instrument::KorioInstrument;
#[doc(inline)]
pub use self::span::Span;
pub use self::{dispatcher::Dispatch, metadata::Severity, subscriber::Subscriber};

#[doc(hidden)]
mod time {
    use std::time::SystemTime;

    pub fn now() -> SystemTime {
        SystemTime::now()
    }
}

#[doc(hidden)]
pub mod __macro_support {
    pub use std::{concat, file, format_args, iter::Iterator, line, option::Option, stringify};
}
