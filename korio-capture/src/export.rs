use std::{
    any::{Any, TypeId},
    fmt,
    marker::PhantomData,
    ops::DerefMut as _,
    sync::Arc,
    time::SystemTime,
};

use futures_channel::mpsc;
use korio::span::Id;
use korio_subscriber::layer::{Context, Observer};
use parking_lot::RwLock;
use tracing::trace;

use crate::CaptureObserver;

pub trait Exporter<Item>: 'static {
    fn close(&self);
    fn export(&self, msg: Item);

    #[doc(hidden)]
    unsafe fn downcast_raw(&self, id: TypeId) -> Option<*const ()> {
        if id == TypeId::of::<Self>() {
            Some(self as *const _ as *const ())
        } else {
            None
        }
    }
}

impl<Item: 'static> dyn Exporter<Item> {
    /// Returns `true` if this `Exporter` is the same type as `T`.
    pub fn is<T: Any>(&self) -> bool {
        self.downcast_ref::<T>().is_some()
    }

    /// Returns some reference to this `Exporter` value if it is of type `T`,
    /// or `None` if it isn't.
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        unsafe {
            let raw = self.downcast_raw(TypeId::of::<T>())?;
            if raw.is_null() {
                None
            } else {
                Some(&*(raw as *const _))
            }
        }
    }
}

impl<Item: 'static> dyn Exporter<Item> + Send {
    /// Returns `true` if this [`Exporter`] is the same type as `T`.
    pub fn is<T: Any>(&self) -> bool {
        self.downcast_ref::<T>().is_some()
    }

    /// Returns some reference to this [`Exporter`] value if it is of type `T`,
    /// or `None` if it isn't.
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        unsafe {
            let raw = self.downcast_raw(TypeId::of::<T>())?;
            if raw.is_null() {
                None
            } else {
                Some(&*(raw as *const _))
            }
        }
    }
}

impl<Item: 'static> dyn Exporter<Item> + Sync {
    /// Returns `true` if this [`Exporter`] is the same type as `T`.
    pub fn is<T: Any>(&self) -> bool {
        self.downcast_ref::<T>().is_some()
    }

    /// Returns some reference to this `[`Exporter`] value if it is of type `T`,
    /// or `None` if it isn't.
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        unsafe {
            let raw = self.downcast_raw(TypeId::of::<T>())?;
            if raw.is_null() {
                None
            } else {
                Some(&*(raw as *const _))
            }
        }
    }
}

impl<Item: 'static> dyn Exporter<Item> + Send + Sync {
    /// Returns `true` if this [`Exporter`] is the same type as `T`.
    pub fn is<T: Any>(&self) -> bool {
        self.downcast_ref::<T>().is_some()
    }

    /// Returns some reference to this [`Exporter`] value if it is of type `T`,
    /// or `None` if it isn't.
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        unsafe {
            let raw = self.downcast_raw(TypeId::of::<T>())?;
            if raw.is_null() {
                None
            } else {
                Some(&*(raw as *const _))
            }
        }
    }
}

#[derive(Debug)]
pub struct Collector<Item>(Arc<RwLock<Vec<Item>>>);

impl<Item> Default for Collector<Item> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<Item> Clone for Collector<Item> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Item> Collector<Item> {
    pub fn take(&self) -> Vec<Item> {
        std::mem::take(self.0.write().deref_mut())
    }
}

impl<Item: 'static> Exporter<Item> for Collector<Item> {
    fn close(&self) {}

    fn export(&self, msg: Item) {
        self.0.write().push(msg);
    }
}

impl<Item: 'static> Exporter<Item> for mpsc::UnboundedSender<Item> {
    fn close(&self) {
        self.close_channel();
    }
    fn export(&self, msg: Item) {
        let _ = self.unbounded_send(msg);
    }
}

struct CloseMarker;

pub struct CloseOnEnd<E, Item> {
    exporter: E,
    _phantom: PhantomData<fn(Item)>,
}

impl<E, Item> fmt::Debug for CloseOnEnd<E, Item>
where
    E: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CloseOnEnd")
            .field("exporter", &self.exporter)
            .finish()
    }
}

impl<Item, E> CloseOnEnd<E, Item>
where
    Item: 'static,
    E: Exporter<Item> + fmt::Debug + 'static,
{
    pub fn with_exporter(exporter: E) -> Self {
        Self {
            exporter,
            _phantom: PhantomData,
        }
    }
}

impl<Item, E> CaptureObserver for CloseOnEnd<E, Item>
where
    Item: 'static,
    E: Exporter<Item> + fmt::Debug + 'static,
{
    fn on_capture(&self, id: &Id, _capture: &crate::Capture, ctx: &dyn Context) {
        trace!(%id, "capturing close");
        let mut span = ctx.span(id).expect("Subscriber is missing provided span");
        span.extensions_mut().insert(CloseMarker);
        trace!(%id, "capturing close done");
    }
}

impl<Item, E> Observer for CloseOnEnd<E, Item>
where
    Item: 'static,
    E: Exporter<Item> + fmt::Debug + 'static,
{
    fn on_end(&self, id: &Id, _end_time: SystemTime, ctx: &dyn Context) {
        trace!(%id, "on_end");
        let mut span = ctx.span(id).expect("Subscriber is missing provided span");
        if span.extensions_mut().remove::<CloseMarker>().is_some() {
            trace!(%id, "on_end actually closing exporter");
            self.exporter.close();
        }
    }
    fn on_close(&self, id: &Id, ctx: &dyn Context) {
        trace!(%id, "on_close");
        let mut span = ctx.span(id).expect("Subscriber is missing provided span");
        if span.extensions_mut().remove::<CloseMarker>().is_some() {
            trace!(%id, "on_close actually closing exporter");
            self.exporter.close();
        }
    }
}
