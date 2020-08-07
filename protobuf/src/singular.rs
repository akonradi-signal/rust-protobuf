#[cfg(feature = "with-serde")]
use serde;

use std::default::Default;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::option;

use crate::clear::Clear;
use crate::Message;

/// Option-like objects
#[doc(hidden)]
pub trait OptionLike<T> {
    fn into_option(self) -> Option<T>;
    fn as_option_ref(&self) -> Option<&T>;
    fn as_option_mut(&mut self) -> Option<&mut T>;
    fn set_value(&mut self, value: T);
    fn set_default(&mut self) -> &mut T
    where
        T: Default + Clear;
}

impl<T> OptionLike<T> for Option<T> {
    fn into_option(self) -> Option<T> {
        self
    }

    fn as_option_ref(&self) -> Option<&T> {
        self.as_ref()
    }

    fn as_option_mut(&mut self) -> Option<&mut T> {
        self.as_mut()
    }

    fn set_value(&mut self, value: T) {
        *self = Some(value);
    }

    fn set_default(&mut self) -> &mut T
    where
        T: Default + Clear,
    {
        if self.is_some() {
            let v = self.as_mut().unwrap();
            v.clear();
            v
        } else {
            *self = Some(Default::default());
            self.as_mut().unwrap()
        }
    }
}

impl<T> OptionLike<T> for Option<Box<T>> {
    fn into_option(self) -> Option<T> {
        self.map(|b| *b)
    }

    fn as_option_ref(&self) -> Option<&T> {
        self.as_ref().map(|b| b.as_ref())
    }

    fn as_option_mut(&mut self) -> Option<&mut T> {
        self.as_mut().map(|b| b.as_mut())
    }

    fn set_value(&mut self, value: T) {
        // TODO: reuse allocation
        *self = Some(Box::new(value))
    }

    fn set_default(&mut self) -> &mut T
    where
        T: Default + Clear,
    {
        if self.is_some() {
            let v = self.as_mut().unwrap();
            v.clear();
            v
        } else {
            *self = Some(Box::new(Default::default()));
            self.as_mut().unwrap()
        }
    }
}

/// Like `Option<Box<T>>`, but keeps the actual element on `clear`.
///
/// # Examples
///
/// ```no_run
/// # use protobuf::SingularPtrField;
/// # use std::ops::Add;
/// # struct Address {
/// # }
/// # struct Customer {
/// #     address: SingularPtrField<Address>,
/// # }
/// # impl Customer {
/// #     fn new() -> Customer { unimplemented!() }
/// # }
/// #
/// #
/// # fn make_address() -> Address { unimplemented!() }
/// let mut customer = Customer::new();
///
/// // field of type `SingularPtrField` can be initialized like this
/// customer.address = SingularPtrField::some(make_address());
/// // or using `Option` and `Into`
/// customer.address = Some(make_address()).into();
/// ```
pub struct SingularPtrField<T> {
    value: Option<Box<T>>,
    set: bool,
}

impl<T> SingularPtrField<T> {
    /// Construct `SingularPtrField` from given object.
    #[inline]
    pub fn some(value: T) -> SingularPtrField<T> {
        SingularPtrField {
            value: Some(Box::new(value)),
            set: true,
        }
    }

    /// Construct an empty `SingularPtrField`.
    #[inline]
    pub const fn none() -> SingularPtrField<T> {
        SingularPtrField {
            value: None,
            set: false,
        }
    }

    /// Construct `SingularPtrField` from optional.
    #[inline]
    pub fn from_option(option: Option<T>) -> SingularPtrField<T> {
        match option {
            Some(x) => SingularPtrField::some(x),
            None => SingularPtrField::none(),
        }
    }

    /// True iff this object contains data.
    #[inline]
    pub fn is_some(&self) -> bool {
        self.set
    }

    /// True iff this object contains no data.
    #[inline]
    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    /// Convert into `Option<T>`.
    #[inline]
    pub fn into_option(self) -> Option<T> {
        if self.set {
            Some(*self.value.unwrap())
        } else {
            None
        }
    }

    /// View data as reference option.
    #[inline]
    pub fn as_ref(&self) -> Option<&T> {
        if self.set {
            Some(&**self.value.as_ref().unwrap())
        } else {
            None
        }
    }

    /// View data as mutable reference option.
    #[inline]
    pub fn as_mut(&mut self) -> Option<&mut T> {
        if self.set {
            Some(&mut **self.value.as_mut().unwrap())
        } else {
            None
        }
    }

    /// Get data as reference.
    /// Panics if empty.
    #[inline]
    pub fn get_ref(&self) -> &T {
        self.as_ref().unwrap()
    }

    /// Get data as mutable reference.
    /// Panics if empty.
    #[inline]
    pub fn get_mut_ref(&mut self) -> &mut T {
        self.as_mut().unwrap()
    }

    /// Take the data.
    /// Panics if empty
    #[inline]
    pub fn unwrap(self) -> T {
        if self.set {
            *self.value.unwrap()
        } else {
            panic!();
        }
    }

    /// Take the data or return supplied default element if empty.
    #[inline]
    pub fn unwrap_or(self, def: T) -> T {
        if self.set {
            *self.value.unwrap()
        } else {
            def
        }
    }

    /// Take the data or return supplied default element if empty.
    #[inline]
    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        if self.set {
            *self.value.unwrap()
        } else {
            f()
        }
    }

    /// Apply given function to contained data to construct another `SingularPtrField`.
    /// Returns empty `SingularPtrField` if this object is empty.
    #[inline]
    pub fn map<U, F>(self, f: F) -> SingularPtrField<U>
    where
        F: FnOnce(T) -> U,
    {
        SingularPtrField::from_option(self.into_option().map(f))
    }

    /// View data as iterator.
    #[inline]
    pub fn iter(&self) -> option::IntoIter<&T> {
        self.as_ref().into_iter()
    }

    /// View data as mutable iterator.
    #[inline]
    pub fn mut_iter(&mut self) -> option::IntoIter<&mut T> {
        self.as_mut().into_iter()
    }

    /// Take data as option, leaving this object empty.
    #[inline]
    pub fn take(&mut self) -> Option<T> {
        if self.set {
            self.set = false;
            Some(*self.value.take().unwrap())
        } else {
            None
        }
    }

    /// Clear this object, but do not call destructor of underlying data.
    #[inline]
    pub fn clear(&mut self) {
        self.set = false;
    }
}

impl<T: Default + Clear> SingularPtrField<T> {
    /// Get contained data, consume self. Return default value for type if this is empty.
    #[inline]
    pub fn unwrap_or_default(mut self) -> T {
        if self.set {
            self.unwrap()
        } else if self.value.is_some() {
            self.value.clear();
            *self.value.unwrap()
        } else {
            Default::default()
        }
    }

    /// Set object to `Some(T::default())`.
    // TODO: inline
    #[inline]
    pub fn set_default(&mut self) -> &mut T {
        OptionLike::set_default(self)
    }
}

impl<M: Message + Default> SingularPtrField<M> {
    /// Get a reference to contained value or a default instance.
    pub fn get_or_default(&self) -> &M {
        self.as_ref().unwrap_or_else(|| M::default_instance())
    }

    /// Get a mutable reference to contained value, initialize if not initialized yet.
    pub fn mut_or_default(&mut self) -> &mut M {
        if self.is_none() {
            self.set_default();
        }
        self.get_mut_ref()
    }
}

impl<T> Default for SingularPtrField<T> {
    #[inline]
    fn default() -> SingularPtrField<T> {
        SingularPtrField::none()
    }
}

impl<T> From<Option<T>> for SingularPtrField<T> {
    fn from(o: Option<T>) -> Self {
        SingularPtrField::from_option(o)
    }
}

impl<T: Clone> Clone for SingularPtrField<T> {
    #[inline]
    fn clone(&self) -> SingularPtrField<T> {
        if self.set {
            SingularPtrField::some(self.as_ref().unwrap().clone())
        } else {
            SingularPtrField::none()
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for SingularPtrField<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_some() {
            write!(f, "Some({:?})", *self.as_ref().unwrap())
        } else {
            write!(f, "None")
        }
    }
}

impl<T: PartialEq> PartialEq for SingularPtrField<T> {
    #[inline]
    fn eq(&self, other: &SingularPtrField<T>) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl<T: Eq> Eq for SingularPtrField<T> {}

impl<T: Hash> Hash for SingularPtrField<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl<'a, T> IntoIterator for &'a SingularPtrField<T> {
    type Item = &'a T;
    type IntoIter = option::IntoIter<&'a T>;

    fn into_iter(self) -> option::IntoIter<&'a T> {
        self.iter()
    }
}

impl<T> OptionLike<T> for SingularPtrField<T> {
    fn into_option(self) -> Option<T> {
        self.into_option()
    }

    fn as_option_ref(&self) -> Option<&T> {
        self.as_ref()
    }

    fn as_option_mut(&mut self) -> Option<&mut T> {
        self.as_mut()
    }

    fn set_value(&mut self, value: T) {
        // TODO: unnecessary malloc
        *self = SingularPtrField::some(value);
    }

    /// Initialize this object with default value.
    /// This operation can be more efficient then construction of clear element,
    /// because it may reuse previously contained object.
    #[inline]
    fn set_default(&mut self) -> &mut T
    where
        T: Default + Clear,
    {
        self.set = true;
        if self.value.is_some() {
            self.value.as_mut().unwrap().clear();
        } else {
            self.value = Some(Default::default());
        }
        self.as_mut().unwrap()
    }
}

#[cfg(feature = "with-serde")]
impl<T: serde::Serialize> serde::Serialize for SingularPtrField<T> {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error>
    where
        S: serde::Serializer,
    {
        self.as_ref().serialize(serializer)
    }
}

#[cfg(feature = "with-serde")]
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for SingularPtrField<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as serde::Deserializer<'de>>::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Option::deserialize(deserializer).map(SingularPtrField::from)
    }
}
