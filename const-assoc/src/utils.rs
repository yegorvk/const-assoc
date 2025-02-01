use core::mem;
use core::mem::MaybeUninit;
use sealed::sealed;
use typewit::HasTypeWitness;

pub trait ConstIntoUSize: HasTypeWitness<IntoUSizeWitness<Self>> {}
impl<T: HasTypeWitness<IntoUSizeWitness<Self>>> ConstIntoUSize for T {}

#[cfg(target_pointer_width = "16")]
#[inline(always)]
pub const fn into_usize<T: ConstIntoUSize>(value: T) -> usize {
    match T::WITNESS {
        IntoUSizeWitness::U8(te) => te.to_right(value) as usize,
        IntoUSizeWitness::U16(te) => te.to_right(value) as usize,
        IntoUSizeWitness::USize(te) => te.to_right(value),
    }
}

#[cfg(target_pointer_width = "16")]
typewit::simple_type_witness! {
    enum IntoUSizeWitness {
        U8 = u8,
        U16 = u16,
        USize = usize,
    }
}

#[cfg(target_pointer_width = "32")]
#[inline(always)]
pub const fn into_usize<T: ConstIntoUSize>(value: T) -> usize {
    match T::WITNESS {
        IntoUSizeWitness::U8(te) => te.to_right(value) as usize,
        IntoUSizeWitness::U16(te) => te.to_right(value) as usize,
        IntoUSizeWitness::U32(te) => te.to_right(value) as usize,
        IntoUSizeWitness::USize(te) => te.to_right(value),
    }
}

#[cfg(target_pointer_width = "32")]
typewit::simple_type_witness! {
    enum IntoUSizeWitness {
        U8 = u8,
        U16 = u16,
        U32 = u32,
        USize = usize,
    }
}

#[cfg(target_pointer_width = "64")]
#[inline(always)]
pub const fn into_usize<T: ConstIntoUSize>(value: T) -> usize {
    match T::WITNESS {
        IntoUSizeWitness::U8(te) => te.to_right(value) as usize,
        IntoUSizeWitness::U16(te) => te.to_right(value) as usize,
        IntoUSizeWitness::U32(te) => te.to_right(value) as usize,
        IntoUSizeWitness::U64(te) => te.to_right(value) as usize,
        IntoUSizeWitness::USize(te) => te.to_right(value),
    }
}

#[cfg(target_pointer_width = "64")]
typewit::simple_type_witness! {
    enum IntoUSizeWitness {
        U8 = u8,
        U16 = u16,
        U32 = u32,
        U64 = u64,
        USize = usize,
    }
}

/// Indicates that transmuting `Self` to `Dst` is always safe.
///
/// # Safety
/// Implementors of this trait must ensure that calling both
/// `mem::transmute::<Self, Dst>` and `mem::transmute_copy::<Self, Dst>` is
/// always safe.
pub unsafe trait TransmuteSafe<Dst: Copy>: Copy {}

#[inline(always)]
pub const fn transmute_safe<Src, Dst>(src: Src) -> Dst
where
    Src: TransmuteSafe<Dst>,
    Dst: Copy,
{
    transmute_copy_safe(&src)
}

#[inline(always)]
pub const fn transmute_copy_safe<Src, Dst>(src: &Src) -> Dst
where
    Src: TransmuteSafe<Dst>,
    Dst: Copy,
{
    unsafe { mem::transmute_copy(src) }
}

pub struct ConstUSize<const N: usize>;

#[sealed]
pub trait IsConstUSize {
    const N: usize;
}

#[sealed]
impl<const N: usize> IsConstUSize for ConstUSize<N> {
    const N: usize = N;
}

#[sealed]
pub trait Is<Rhs> {}

#[sealed]
impl<T> Is<T> for T {}

#[inline(always)]
pub const unsafe fn assume_init_array<T, const N: usize>(array: [MaybeUninit<T>; N]) -> [T; N] {
    // This code heavily relies on the compiler understanding that this is a no-op.
    // Unfortunately, we can't directly use transmute here, because the compiler
    // cannot prove that `[MaybeUninit<T>; N]` has the same size as `[T; N]`
    let array = MaybeUninit::new(array);
    let ptr: *const [T; N] = array.as_ptr() as *const [T; N];
    unsafe { ptr.cast::<[T; N]>().read() }
}
