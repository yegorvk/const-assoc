#![allow(private_bounds)]

mod utils;

use crate::utils::{
    into_usize, transmute_safe, ConstIntoUSize, ConstUSize, Is, IsConstUSize, TransmuteSafe,
};
use derive_where::derive_where;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

// Re-export `const_default::ConstDefault`.
pub use const_default::ConstDefault;

#[macro_export]
macro_rules! const_array_map {
    ($($key:expr => $value:expr),* $(,)?) => {
        {
            let mut map = <$crate::ConstArrayMap<_, _> as $crate::ConstDefault>::DEFAULT;
            $( *map.get_mut($key) = $value; )*
            map
        }
    };
}

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
    #[inline(always)]
    pub fn get(&self, key: K) -> &V {
        let idx = key_to_index(transmute_safe(key));
        // SAFETY: The invariant of `KeyImpl` guarantees that
        // `idx` is always less than `self.storage.len()`.
        unsafe { self.storage.get_unchecked(idx) }
    }

    #[inline(always)]
    pub const fn const_get(&self, key: K) -> &V {
        let idx = key_to_index(transmute_safe(key));
        &self.storage[idx]
    }

    #[inline(always)]
    pub fn get_mut(&mut self, key: K) -> &mut V {
        let idx = key_to_index(transmute_safe(key));
        // SAFETY: The invariant of `KeyImpl` guarantees that
        // `idx` is always less than `self.storage.len()`.
        unsafe { self.storage.get_unchecked_mut(idx) }
    }

    #[inline(always)]
    pub const fn const_get_mut(&mut self, key: K) -> &mut V {
        let idx = key_to_index(transmute_safe(key));
        &mut self.storage[idx]
    }

    pub fn into_values(self) -> impl Iterator<Item = V> {
        self.storage.into_iter()
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.storage.iter()
    }

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
unsafe impl<T: PrimitiveEnum, _U> TransmuteSafe<<T::Layout as EnumLayoutTrait>::Discriminant>
    for EnumKeyImpl<T, _U>
{
}

impl<T: PrimitiveEnum> Key for T
where
    EnumKeyImpl<T, <<T as PrimitiveEnum>::Layout as EnumLayoutTrait>::Variants>: KeyImpl,
{
    type Impl = EnumKeyImpl<T, <T::Layout as EnumLayoutTrait>::Variants>;
}

/// Indicates that `Self` is a primitive enum type, meaning that it is an enum
/// with a `#[repr(primitive_type)]` attribute.
///
/// # Safety
/// The implementors must ensure that `Layout` exactly describes `Self`.
pub unsafe trait PrimitiveEnum: Copy {
    type Layout: EnumLayoutTrait;
}

// SAFETY: The invariant of `PrimitiveEnum` implies that `Self` always represents
// a valid enum discriminant when converted to usize, so it must be an integer
// ranging from 0 to `VARIANTS`-1.
unsafe impl<T: PrimitiveEnum, const VARIANTS: usize> KeyImpl
    for EnumKeyImpl<T, ConstUSize<VARIANTS>>
where
    <<T as PrimitiveEnum>::Layout as EnumLayoutTrait>::Variants: Is<ConstUSize<VARIANTS>>,
    EnumKeyImpl<T, ConstUSize<VARIANTS>>:
        TransmuteSafe<<<T as PrimitiveEnum>::Layout as EnumLayoutTrait>::Discriminant>,
{
    type Storage<V> = [V; VARIANTS];
    type Repr = <<T as PrimitiveEnum>::Layout as EnumLayoutTrait>::Discriminant;
}

trait EnumLayoutTrait {
    type Discriminant: Copy + ConstIntoUSize;
    type Variants: IsConstUSize;
}

pub struct EnumLayout<Discriminant, const VARIANTS: usize> {
    _marker: PhantomData<Discriminant>,
}

impl<Discriminant, const VARIANTS: usize> EnumLayoutTrait for EnumLayout<Discriminant, VARIANTS>
where
    Discriminant: Copy + ConstIntoUSize,
{
    type Discriminant = Discriminant;
    type Variants = ConstUSize<VARIANTS>;
}

#[cfg(test)]
mod tests {
    use crate::{EnumLayout, PrimitiveEnum};

    macro_rules! enum_tests {
        ($($test_name:ident => $repr:ty),* $(,)?) => {
            $(
                #[test]
                fn $test_name() {
                    #[repr($repr)]
                    #[derive(Copy, Clone)]
                    enum Letter {
                        A,
                        B,
                        C,
                        D,
                    }

                    unsafe impl PrimitiveEnum for Letter {
                        type Layout = EnumLayout<$repr, 4>;
                    }

                    let mut letters = const_array_map! {
                        Letter::A => 'a',
                        Letter::B => 'b',
                        Letter::C => 'c',
                        Letter::D => 'd',
                    };

                    assert_eq!(letters[Letter::A], 'a');
                    assert_eq!(*letters.get(Letter::B), 'b');
                    assert_eq!(*letters.const_get(Letter::C), 'c');

                    letters[Letter::B] = 'x';
                    assert_eq!(letters[Letter::B], 'x');
                }
            )*
        };
    }

    #[cfg(target_pointer_width = "16")]
    enum_tests! {
        enum_u8 => u8,
        enum_u16 => u16,
        enum_usize => usize,
    }

    #[cfg(target_pointer_width = "32")]
    enum_tests! {
        enum_u8 => u8,
        enum_u16 => u16,
        enum_u32 => u32,
        enum_usize => usize,
    }

    #[cfg(target_pointer_width = "64")]
    enum_tests! {
        enum_u8 => u8,
        enum_u16 => u16,
        enum_u32 => u32,
        enum_u64 => u64,
        enum_usize => usize,
    }
}
