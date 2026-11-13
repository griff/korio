#![allow(clippy::needless_lifetimes, clippy::extra_unused_type_parameters)]

pub mod capture_capnp {
    include!(concat!(env!("OUT_DIR"), "/capture_capnp.rs"));
}

pub mod common_capnp {
    include!(concat!(env!("OUT_DIR"), "/common_capnp.rs"));
}

pub mod event_capnp {
    include!(concat!(env!("OUT_DIR"), "/event_capnp.rs"));
}

pub mod trace_capnp {
    include!(concat!(env!("OUT_DIR"), "/trace_capnp.rs"));
}
