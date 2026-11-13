use std::{borrow::Borrow, fmt, ops::Deref};

use smol_str::SmolStr;

/// Key for attribute.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Key(str);

impl Key {
    /// Return [`Key`] with privided name.
    ///
    /// # Panics
    /// This will panic if the provided name is not valid.
    pub const fn new(name: &str) -> &Key {
        // SAFETY: `str` and `Key` have the same ABI
        unsafe { &*(name as *const str as *const Key) }
    }

    /// Return the name of this key.
    pub const fn as_str(&self) -> &str {
        &self.0
    }

    /// Return the name of this key.
    pub fn as_key(&self) -> &Key {
        self
    }
}

impl AsRef<str> for Key {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq<Key> for &Key {
    fn eq(&self, other: &Key) -> bool {
        **self == *other
    }
}

impl PartialEq<&str> for &Key {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<&str> for Key {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<Key> for &str {
    fn eq(&self, other: &Key) -> bool {
        *self == other.as_str()
    }
}

impl ToOwned for Key {
    type Owned = KeyBuf;

    fn to_owned(&self) -> Self::Owned {
        KeyBuf(SmolStr::new(&self.0))
    }
}

/// Owned key for attribute
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct KeyBuf(SmolStr);

impl KeyBuf {
    /// Return [`Key`] with privided name.
    ///
    /// # Panics
    /// This will panic if the provided name is not valid.
    pub const fn from_static(name: &'static str) -> Self {
        Self(SmolStr::new_static(name))
    }

    /// Return [`KeyBuf`] with privided name if valid
    pub fn new<A: AsRef<str>>(name: A) -> Self {
        Self(SmolStr::new(name))
    }

    /// Return the name of this key.
    pub fn as_key(&self) -> &Key {
        Key::new(&self.0)
    }
}

impl From<String> for KeyBuf {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

impl From<SmolStr> for KeyBuf {
    fn from(name: SmolStr) -> Self {
        Self(name)
    }
}

impl AsRef<Key> for KeyBuf {
    fn as_ref(&self) -> &Key {
        self.as_key()
    }
}

impl Deref for KeyBuf {
    type Target = Key;

    fn deref(&self) -> &Self::Target {
        self.as_key()
    }
}

impl Borrow<Key> for KeyBuf {
    fn borrow(&self) -> &Key {
        self.as_key()
    }
}

/// Trait implemented to allow a type to be used as a `Key`.
pub trait AsKey {
    /// Attempt to convert `&self` into a `Key` and return `None` if not possible.
    fn as_key(&self) -> Option<&Key>;
}

impl AsKey for Key {
    fn as_key(&self) -> Option<&Key> {
        Some(self)
    }
}

impl AsKey for KeyBuf {
    fn as_key(&self) -> Option<&Key> {
        Some(self)
    }
}

impl AsKey for str {
    fn as_key(&self) -> Option<&Key> {
        Some(Key::new(self))
    }
}

/// Formats arguments to a `KeyBuf`.
#[macro_export]
macro_rules! format_key {
    ($($t:tt)*) => {
        $crate::metadata::KeyBuf::from($crate::_exported::smol_str::format_smolstr!($($t)*))
    };
}
