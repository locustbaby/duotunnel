use serde::{Deserialize, Serialize};
use std::sync::Arc;

use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
)]
#[serde(transparent)]
pub struct ClientId(pub Arc<str>);

impl ClientId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for ClientId {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for ClientId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<str> for ClientId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Arc<str>> for ClientId {
    fn from(s: Arc<str>) -> Self {
        Self(s)
    }
}

impl From<String> for ClientId {
    fn from(s: String) -> Self {
        Self(s.into())
    }
}

impl From<&str> for ClientId {
    fn from(s: &str) -> Self {
        Self(s.into())
    }
}

impl Default for ClientId {
    fn default() -> Self {
        Self("".into())
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
)]
#[serde(transparent)]
pub struct GroupId(pub Arc<str>);

impl GroupId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for GroupId {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for GroupId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<str> for GroupId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for GroupId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Arc<str>> for GroupId {
    fn from(s: Arc<str>) -> Self {
        Self(s)
    }
}

impl From<String> for GroupId {
    fn from(s: String) -> Self {
        Self(s.into())
    }
}

impl From<&str> for GroupId {
    fn from(s: &str) -> Self {
        Self(s.into())
    }
}

impl Default for GroupId {
    fn default() -> Self {
        Self("".into())
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
)]
#[serde(transparent)]
pub struct ProxyName(pub Arc<str>);

impl ProxyName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for ProxyName {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for ProxyName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<str> for ProxyName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ProxyName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Arc<str>> for ProxyName {
    fn from(s: Arc<str>) -> Self {
        Self(s)
    }
}

impl From<String> for ProxyName {
    fn from(s: String) -> Self {
        Self(s.into())
    }
}

impl From<&str> for ProxyName {
    fn from(s: &str) -> Self {
        Self(s.into())
    }
}

impl Default for ProxyName {
    fn default() -> Self {
        Self("".into())
    }
}

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    Default,
)]
pub struct ReuseHash(pub u64);

impl std::ops::Deref for ReuseHash {
    type Target = u64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for ReuseHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for ReuseHash {
    fn from(v: u64) -> Self {
        Self(v)
    }
}

macro_rules! impl_partial_eq {
    ($type:ty) => {
        impl PartialEq<str> for $type {
            fn eq(&self, other: &str) -> bool {
                &*self.0 == other
            }
        }
        impl PartialEq<$type> for str {
            fn eq(&self, other: &$type) -> bool {
                self == &*other.0
            }
        }
        impl<'a> PartialEq<&'a str> for $type {
            fn eq(&self, other: &&'a str) -> bool {
                &*self.0 == *other
            }
        }
        impl<'a> PartialEq<$type> for &'a str {
            fn eq(&self, other: &$type) -> bool {
                *self == &*other.0
            }
        }
        impl PartialEq<String> for $type {
            fn eq(&self, other: &String) -> bool {
                &*self.0 == other.as_str()
            }
        }
        impl PartialEq<$type> for String {
            fn eq(&self, other: &$type) -> bool {
                self.as_str() == &*other.0
            }
        }
    };
}

impl_partial_eq!(ClientId);
impl_partial_eq!(GroupId);
impl_partial_eq!(ProxyName);
