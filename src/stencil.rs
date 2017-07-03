use num_traits::Num;
use ::behavior::ThreadSafeCopyable;

/// Defines a type that can be used in a stencil buffer.
///
/// This is different from `num_traits` `PrimInt` and so forth, because a no-op implementation exists for `()`
///
/// This is automatically implemented for any type that implements its dependent traits
pub trait Stencil: ThreadSafeCopyable + Default + PartialOrd {
    /// Equivalent to `Zero::zero()`
    fn zero() -> Self;
    /// Equivalent to `One::one()`
    fn one() -> Self;
    /// Equivalent to `Not::not`
    fn not(self) -> Self;
    /// Equivalent to `wrapping_add` on primitive integers
    fn wrapping_add(self, rhs: Self) -> Self;
    /// Equivalent to `wrapping_sub` on primitive integers
    fn wrapping_sub(self, rhs: Self) -> Self;
    /// Equivalent to `saturating_add` on primitive integers
    fn saturating_add(self, rhs: Self) -> Self;
    /// Equivalent to `saturating_sub` on primitive integers
    fn saturating_sub(self, rhs: Self) -> Self;
}

macro_rules! impl_stencil {
    ($($t:ty),+) => {
        $(
            impl Stencil for $t {
                #[inline(always)]
                fn zero() -> $t { 0 }

                #[inline(always)]
                fn one() -> $t { 1 }

                #[inline(always)]
                fn not(self) -> $t { !self }

                #[inline(always)]
                fn wrapping_add(self, rhs: $t) -> $t {
                    <$t>::wrapping_add(self, rhs)
                }

                #[inline(always)]
                fn wrapping_sub(self, rhs: $t) -> $t {
                    <$t>::wrapping_sub(self, rhs)
                }

                #[inline(always)]
                fn saturating_add(self, rhs: $t) -> $t {
                    <$t>::saturating_add(self, rhs)
                }

                #[inline(always)]
                fn saturating_sub(self, rhs: $t) -> $t {
                    <$t>::saturating_sub(self, rhs)
                }
            }
        )+
    }
}

impl_stencil!(u8, u16, u32, u64, i8, i16, i32, i64, usize, isize);

impl Stencil for () {
    #[inline(always)]
    fn zero() -> Self { () }

    #[inline(always)]
    fn one() -> Self { () }

    #[inline(always)]
    fn not(self) -> Self { () }

    #[inline(always)]
    fn wrapping_add(self, _: Self) -> Self { () }

    #[inline(always)]
    fn wrapping_sub(self, _: Self) -> Self { () }

    #[inline(always)]
    fn saturating_add(self, _: Self) -> Self { () }

    #[inline(always)]
    fn saturating_sub(self, _: Self) -> Self { () }
}

/// Defines tests which can be performed on stencil buffers
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum StencilTest {
    /// Always pass
    Always,
    /// Never pass
    Never,
    /// Pass if the new value is less than the previous
    LessThan,
    /// Pass if the new value is greater than the previous
    GreaterThan,
    /// Pass if the new value is less than or equal to the previous
    LessThanEq,
    /// Pass if the new value is greater than or equal to the previous
    GreaterThanEq,
    /// Pass only if the new value is equal to the previous
    Equal,
    /// Pass only if the new value is NOT equal to the previous
    NotEqual,
}

impl StencilTest {
    /// Performs the stencil test on any `StencilType` type
    #[inline]
    pub fn test<T>(&self, present: T, value: T) -> bool where T: Stencil {
        match *self {
            StencilTest::Always => true,
            StencilTest::Never => false,
            StencilTest::LessThan => value < present,
            StencilTest::LessThanEq => value <= present,
            StencilTest::GreaterThan => value > present,
            StencilTest::GreaterThanEq => value >= present,
            StencilTest::Equal => value == present,
            StencilTest::NotEqual => value != present,
        }
    }
}


/// Defines the operation to be performed upon a passing stencil test
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum StencilOp<T: Stencil> {
    /// Keep the previous stencil value
    Keep,
    /// Perform a bitwise negation on the previous value
    Invert,
    /// Replace the previous value with zero
    Zero,
    /// Replace the previous value with the given value
    Replace(T),
    /// Increment the previous value by one, wrapping as desired.
    Increment { wrap: bool },
    /// Decrement the previous value by one, wrapping as desired.
    Decrement { wrap: bool },
}

impl<T> StencilOp<T> where T: Stencil {
    /// Performs the operation on the value, returning the new value
    #[inline]
    pub fn op(&self, value: T) -> T {
        match *self {
            StencilOp::Keep => value,
            StencilOp::Invert => Stencil::not(value),
            StencilOp::Zero => Stencil::zero(),
            StencilOp::Replace(replacement) => replacement,
            StencilOp::Increment { wrap: true } => Stencil::wrapping_add(value, Stencil::one()),
            StencilOp::Decrement { wrap: true } => Stencil::wrapping_sub(value, Stencil::one()),
            StencilOp::Increment { wrap: false } => Stencil::saturating_add(value, Stencil::one()),
            StencilOp::Decrement { wrap: false } => Stencil::saturating_sub(value, Stencil::one()),
        }
    }
}

/// Defines a stateful configuration for a stencil buffer
pub trait StencilConfig<T: Stencil>: Clone + Copy + Default {
    /// Return the operation to be performed
    fn op(&self) -> StencilOp<T>;
    /// Return the test to be performed
    fn test(&self) -> StencilTest;
}

impl StencilConfig<()> for () {
    #[inline(always)]
    fn op(&self) -> StencilOp<()> { StencilOp::Keep }

    #[inline(always)]
    fn test(&self) -> StencilTest { StencilTest::Always }
}

/// Generic stencil config that just stores the `StencilOp` and `StencilTest` structures.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GenericStencilConfig<T: Stencil = ()> {
    pub op: StencilOp<T>,
    pub test: StencilTest,
}

impl<T> StencilConfig<T> for GenericStencilConfig<T> where T: Stencil {
    #[inline(always)]
    fn op(&self) -> StencilOp<T> { self.op }

    #[inline(always)]
    fn test(&self) -> StencilTest { self.test }
}

impl<T> Default for GenericStencilConfig<T> where T: Stencil {
    fn default() -> GenericStencilConfig<T> {
        GenericStencilConfig {
            op: StencilOp::Keep,
            test: StencilTest::Always,
        }
    }
}
