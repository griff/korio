#![allow(clippy::needless_lifetimes, clippy::extra_unused_type_parameters)]

pub mod koriosc_capnp {
    include!(concat!(env!("OUT_DIR"), "/koriosc_capnp.rs"));
}
