use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

/// Collection of extensions
#[derive(Debug, Default, Clone)]
pub struct ExtensionSet {
    inner: Arc<RwLock<ExtensionsInner>>,
}

impl ExtensionSet {
    /// Get immutable access to extensions.
    pub fn read(&self) -> Extensions<'_> {
        let inner = self.inner.read().expect("bug: rwlock has been poisoned");
        Extensions { inner }
    }

    /// Get mutable access to extensions.
    pub fn write(&self) -> ExtensionsMut<'_> {
        let inner = self.inner.write().expect("bug: rwlock has been poisoned");
        ExtensionsMut { inner }
    }
}

#[derive(Debug, Default)]
struct ExtensionsInner {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ExtensionsInner {
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|val| val.downcast_ref())
    }

    pub fn replace<T: 'static + Send + Sync>(&mut self, value: T) -> Option<T> {
        self.map
            .insert(TypeId::of::<T>(), Box::new(value))
            .and_then(|old| old.downcast().ok())
            .map(|old_t| *old_t)
    }

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.map
            .get_mut(&TypeId::of::<T>())
            .and_then(|val| val.downcast_mut())
    }

    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        self.map
            .remove(&TypeId::of::<T>())
            .and_then(|old| old.downcast().ok())
            .map(|old_t| *old_t)
    }
}

/// Immutable access to extensions.
#[derive(Debug)]
pub struct Extensions<'e> {
    inner: RwLockReadGuard<'e, ExtensionsInner>,
}

impl Extensions<'_> {
    /// Get `T` extension.
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.inner.get()
    }
}

/// Mutable access to extensions.
#[derive(Debug)]
pub struct ExtensionsMut<'e> {
    inner: RwLockWriteGuard<'e, ExtensionsInner>,
}

impl ExtensionsMut<'_> {
    /// Insert `T` extension.
    ///
    /// **panic:** This will panic if there already exists an extension `T`.s
    pub fn insert<T: 'static + Send + Sync>(&mut self, value: T) {
        assert!(self.replace(value).is_none());
    }

    /// Replace extension `T` with new value.
    ///
    /// Returns any old extension value of type `T`.
    pub fn replace<T: 'static + Send + Sync>(&mut self, value: T) -> Option<T> {
        self.inner.replace(value)
    }

    /// Get mutable access to extension `T` if it exists.
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.inner.get_mut()
    }

    /// Remove extenion `T` and return value.
    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        self.inner.remove()
    }
}
