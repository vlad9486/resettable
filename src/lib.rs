// Copyright 2021 Vladislav Melnik
// SPDX-License-Identifier: MIT

#![forbid(unsafe_code)]

#[cfg(feature = "derive")]
pub use resettable_derive::*;

use std::{
    ops::{Deref, DerefMut},
    fmt,
};

pub trait Resettable {
    fn reset(self) -> Self;
}

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResettableWrapper<T> {
    inner: T,
    stash: Option<T>,
}

impl<T> Deref for ResettableWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for ResettableWrapper<T>
where
    T: Clone,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        if self.stash.is_none() {
            self.stash = Some(self.inner.clone());
        }

        &mut self.inner
    }
}

impl<T> ResettableWrapper<T> {
    pub fn new(inner: T) -> Self {
        ResettableWrapper { inner, stash: None }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn reset_inner(self) -> T {
        if let Some(stash) = self.stash {
            stash
        } else {
            self.inner
        }
    }
}

impl<T> From<T> for ResettableWrapper<T> {
    fn from(inner: T) -> Self {
        ResettableWrapper::new(inner)
    }
}

impl<T> Resettable for ResettableWrapper<T> {
    fn reset(self) -> Self {
        ResettableWrapper::new(self.reset_inner())
    }
}

impl<T> fmt::Debug for ResettableWrapper<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

#[cfg(feature = "serde")]
mod serde_m {
    use serde::{ser, de};

    use super::ResettableWrapper;

    impl<T> ser::Serialize for ResettableWrapper<T>
    where
        T: ser::Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
        {
            self.inner.serialize(serializer)
        }
    }

    impl<'de, T> de::Deserialize<'de> for ResettableWrapper<T>
    where
        T: de::Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            T::deserialize(deserializer).map(ResettableWrapper::new)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ResettableWrapper, Resettable};

    #[test]
    fn basic() {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
        struct Complex {
            first: ResettableWrapper<usize>,
            second: ResettableWrapper<String>,
        }

        impl Resettable for Complex {
            fn reset(self) -> Self {
                Complex {
                    first: self.first.reset(),
                    second: self.second.reset(),
                }
            }
        }

        let original = Complex {
            first: 123.into(),
            second: "foo".to_string().into(),
        };
        let mut complex = original.clone();
        println!("original: {:?}", complex);

        *complex.first += 100;
        println!("mutated first: {:?}", complex);

        let mut complex = complex.reset();

        *complex.second = "bar".to_string();
        println!("mutated second: {:?}", complex);

        let reset = complex.reset();
        println!("original: {:?}", reset);

        assert_eq!(reset, original);
    }
}
