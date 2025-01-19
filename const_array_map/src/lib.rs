#![no_std]
#![allow(private_bounds)]

//! A `no_std`-compatible, const-capable Map type backed by an array.
//!
//! This crate defines a new map type, [`ConstArrayMap`], similar to other
//! data structures but implemented using a single array with
//! zero-cost conversion between keys and array indices.
//!
//! Currently, keys are limited to enums with a primitive representation. In the future,
//! it might also be possible to support arbitrary types with a relatively small
//! number of distinct valid values, possibly at the expense of not exposing
//! `const`-qualified methods for these key types.
//!
//! # Example
//! ```
//! use const_array_map::{const_array_map, PrimitiveEnum};
//!
//! #[repr(u8)]
//! #[derive(Copy, Clone, PrimitiveEnum)]
//! enum Letter {
//!     A,
//!     B,
//!     C,
//! }
//!
//! let letters = const_array_map! {
//!     Letter::A => 'a',
//!     Letter::B => 'b',
//!     Letter::C => 'c',
//! };
//!
//! assert_eq!(letters[Letter::A], 'a');
//! assert_eq!(letters[Letter::C], 'c');
//! ```

mod utils;

use crate::utils::{
    into_usize, transmute_safe, ConstIntoUSize, ConstUSize, Is, IsConstUSize, TransmuteSafe,
};
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};
use derive_where::derive_where;

// Re-export `const_default::ConstDefault`.
pub use const_default::ConstDefault;

// Re-export the derive macro for `PrimitiveEnum`.
pub use const_array_map_derive::PrimitiveEnum;

/// Provides an easy way to construct a new [`ConstArrayMap`] from a number of
/// key-value pairs that can also be used in const contexts.
#[macro_export]
macro_rules! const_array_map {
    ($($key:expr => $value:expr),* $(,)?) => {
        {
            let mut map = <$crate::ConstArrayMap<_, _> as $crate::ConstDefault>::DEFAULT;
            $( *map.const_get_mut($key) = $value; )*
            map
        }
    };
}

/// An associative array (Map) backed by a Rust's built-in array.
///
/// When used with key types implementing [`PrimitiveEnum`], this type is a
/// zero-cost abstraction over an array as enum variants can be converted
/// to array indices via single transmute and a static cast (`as` operator).
#[repr(transparent)]
pub struct ConstArrayMap<K: Key, V> {
    storage: <K::Impl as KeyImpl>::Storage<V>,
}

impl<K: Key, V: ConstDefault> ConstDefault for ConstArrayMap<K, V>
where
    <K::Impl as KeyImpl>::Storage<V>: ConstDefault,
{
    const DEFAULT: Self = Self {
        storage: <<K::Impl as KeyImpl>::Storage<V>>::DEFAULT,
    };
}

impl<K: Key, V: ConstDefault> Default for ConstArrayMap<K, V>
where
    <K::Impl as KeyImpl>::Storage<V>: Default,
{
    fn default() -> Self {
        Self {
            storage: <<K::Impl as KeyImpl>::Storage<V> as Default>::default(),
        }
    }
}

impl<K: Key, V, const N: usize> ConstArrayMap<K, V>
where
    K::Impl: KeyImpl<Storage<V> = [V; N]>,
{
    /// Returns a reference to the value associated with the given key.
    #[inline(always)]
    pub fn get(&self, key: K) -> &V {
        let idx = key_to_index(transmute_safe(key));
        // SAFETY: The invariant of `KeyImpl` guarantees that
        // `idx` is always less than `self.storage.len()`.
        unsafe { self.storage.get_unchecked(idx) }
    }

    /// Returns a reference to the value associated with the given key.
    /// 
    /// This version does bounds-checking therefore can be used in const 
    /// contexts, unlike `get`.
    #[inline(always)]
    pub const fn const_get(&self, key: K) -> &V {
        let idx = key_to_index(transmute_safe(key));
        &self.storage[idx]
    }

    /// Returns a mutable reference to the value associated with the given key.
    #[inline(always)]
    pub fn get_mut(&mut self, key: K) -> &mut V {
        let idx = key_to_index(transmute_safe(key));
        // SAFETY: The invariant of `KeyImpl` guarantees that
        // `idx` is always less than `self.storage.len()`.
        unsafe { self.storage.get_unchecked_mut(idx) }
    }

    /// Returns a mutable reference to the value associated with the given key.
    ///
    /// This version does bounds-checking, therefore can be used in const 
    /// contexts, unlike `get`.
    #[inline(always)]
    pub const fn const_get_mut(&mut self, key: K) -> &mut V {
        let idx = key_to_index(transmute_safe(key));
        &mut self.storage[idx]
    }

    /// Takes `self` by value and returns an iterator over all the values
    /// stored in this map.
    #[inline]
    pub fn into_values(self) -> impl Iterator<Item = V> {
        self.storage.into_iter()
    }

    /// Returns an iterator over shared references to all values stored in this
    /// map in arbitrary order.
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.storage.iter()
    }

    /// Returns an iterator over mutable references to all values stored in this 
    /// map in arbitrary order.
    #[inline]
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.storage.iter_mut()
    }
}

impl<K: Key, V, const N: usize> Index<K> for ConstArrayMap<K, V>
where
    K::Impl: KeyImpl<Storage<V> = [V; N]>,
{
    type Output = V;

    #[inline(always)]
    fn index(&self, index: K) -> &Self::Output {
        self.get(index)
    }
}

impl<K: Key, V, const N: usize> IndexMut<K> for ConstArrayMap<K, V>
where
    K::Impl: KeyImpl<Storage<V> = [V; N]>,
{
    #[inline(always)]
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        self.get_mut(index)
    }
}

/// Describes a key type for [ConstArrayMap].
///
/// # Safety
/// Whenever `Storage<V>` is an array `[V; N]` for some N, `Self` must be less
/// than `N` when converted to `usize` via
/// `into_usize(transmute_safe<Self, Self::Repr>())`.
unsafe trait KeyImpl: Copy + TransmuteSafe<Self::Repr> {
    type Storage<V>;
    type Repr: Copy + ConstIntoUSize;
}

#[inline(always)]
const fn key_to_index<K: KeyImpl>(key: K) -> usize {
    let repr: K::Repr = transmute_safe(key);
    into_usize(repr)
}

trait Key: TransmuteSafe<Self::Impl> {
    type Impl: KeyImpl;
}

#[repr(transparent)]
#[derive_where(Copy, Clone)]
struct EnumKeyImpl<T: PrimitiveEnum, _U>(T, PhantomData<_U>);

// SAFETY: `EnumKey<T>` has the same representation as `T`.
unsafe impl<T: PrimitiveEnum, _U> TransmuteSafe<EnumKeyImpl<T, _U>> for T {}

// SAFETY: `EnumKey<T>` has the same representation as `T`, while `T` has the
// same representation as `<T::Layout as EnumLayoutTrait>::Discriminant`, since
// `PrimitiveEnum` implies `TransmuteSafe<<T::Layout as EnumLayoutTrait>::Discriminant>`.
unsafe impl<T: PrimitiveEnum, _U>
    TransmuteSafe<<T::Layout as PrimitiveEnumLayoutTrait>::Discriminant> for EnumKeyImpl<T, _U>
{
}

impl<T: PrimitiveEnum> Key for T
where
    EnumKeyImpl<T, <<T as PrimitiveEnum>::Layout as PrimitiveEnumLayoutTrait>::MaxVariants>:
        KeyImpl,
{
    type Impl = EnumKeyImpl<T, <T::Layout as PrimitiveEnumLayoutTrait>::MaxVariants>;
}

/// Indicates that `Self` is a primitive enum type, meaning that it is an enum
/// with a `#[repr(primitive_type)]` attribute.
///
/// # Safety
/// The implementors must ensure that `Layout` exactly describes `Self`.
pub unsafe trait PrimitiveEnum: Copy {
    /// The layout of `Self`.
    type Layout: PrimitiveEnumLayoutTrait;
}

// SAFETY: The invariant of `PrimitiveEnum` implies that `Self` always
// represents a valid enum discriminant when converted to usize, so it must be
// a non-negative integer that is less than `MAX_VARIANTS`.
unsafe impl<T: PrimitiveEnum, const MAX_VARIANTS: usize> KeyImpl
    for EnumKeyImpl<T, ConstUSize<MAX_VARIANTS>>
where
    <<T as PrimitiveEnum>::Layout as PrimitiveEnumLayoutTrait>::MaxVariants:
        Is<ConstUSize<MAX_VARIANTS>>,
    EnumKeyImpl<T, ConstUSize<MAX_VARIANTS>>:
        TransmuteSafe<<<T as PrimitiveEnum>::Layout as PrimitiveEnumLayoutTrait>::Discriminant>,
{
    type Storage<V> = [V; MAX_VARIANTS];
    type Repr = <<T as PrimitiveEnum>::Layout as PrimitiveEnumLayoutTrait>::Discriminant;
}

// See `PrimitiveEnumLayout`.
trait PrimitiveEnumLayoutTrait {
    type Discriminant: Copy + ConstIntoUSize;
    type MaxVariants: IsConstUSize;
}

/// Describes the layout of an enum with a `#[repr(primitive_type)]` attribute.
///
/// # Parameters
/// * `Discriminant` - The underlying numerical type used to represent enum variants.
/// * `MAX_MAX_VARIANTS` - The maximum number of variants this enum can have, equal to
///   the greatest discriminant value among the enum's variants plus 1.
pub struct PrimitiveEnumLayout<Discriminant, const MAX_MAX_VARIANTS: usize> {
    _marker: PhantomData<Discriminant>,
}

impl<Discriminant, const MAX_VARIANTS: usize> PrimitiveEnumLayoutTrait
    for PrimitiveEnumLayout<Discriminant, MAX_VARIANTS>
where
    Discriminant: Copy + ConstIntoUSize,
{
    type Discriminant = Discriminant;
    type MaxVariants = ConstUSize<MAX_VARIANTS>;
}
