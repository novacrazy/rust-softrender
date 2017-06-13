//! Stencil Buffer attachment definitions and operations

use num_traits::{PrimInt, WrappingSub, WrappingAdd};

/// Defines a type that can be used in a stencil buffer.
///
/// This is automatically implemented for any type that implements its dependenct traits
pub trait StencilType: super::Attachment + PrimInt + WrappingSub + WrappingAdd {}

impl<T> StencilType for T where T: super::Attachment + PrimInt + WrappingSub + WrappingAdd {}

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
    pub fn test<T>(&self, present: T, value: T) -> bool where T: StencilType {
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
pub enum StencilOp<T> {
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

impl<T> StencilOp<T> where T: StencilType {
    /// Performs the operation on the value, returning the new value
    #[inline]
    pub fn op(&self, value: T) -> T {
        match *self {
            StencilOp::Keep => value,
            StencilOp::Invert => !value,
            StencilOp::Zero => T::zero(),
            StencilOp::Replace(replacement) => replacement,
            StencilOp::Increment { wrap: true } => value.wrapping_add(&T::one()),
            StencilOp::Decrement { wrap: true } => value.wrapping_sub(&T::one()),
            StencilOp::Increment { wrap: false } => value.saturating_add(T::one()),
            StencilOp::Decrement { wrap: false } => value.saturating_sub(T::one()),
        }
    }
}

/// Defines a stateful configuration for a stencil buffer
pub trait StencilConfig<T> {
    /// Return the operation to be performed
    fn op(&self) -> StencilOp<T>;
    /// Return the test to be performed
    fn test(&self) -> StencilTest;
}

/// Defines a stencil buffer attachment
pub trait Stencil: super::Attachment {
    /// The inner data type of the stencil buffer
    type Type;
    /// The configuration type for the stencil buffer. This has only one instance per attachment.
    type Config: StencilConfig<Self::Type>;
}

impl StencilConfig<()> for () {
    #[inline(always)]
    fn op(&self) -> StencilOp<()> { StencilOp::Keep }

    #[inline(always)]
    fn test(&self) -> StencilTest { StencilTest::Always }
}

impl Stencil for () {
    type Type = ();
    type Config = ();
}

/// Generic stencil config that just stores the `StencilOp` and `StencilTest` structures.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GenericStencilConfig<T> {
    pub op: StencilOp<T>,
    pub test: StencilTest,
}

impl<T> StencilConfig<T> for GenericStencilConfig<T> where T: StencilType {
    #[inline(always)]
    fn op(&self) -> StencilOp<T> { self.op }

    #[inline(always)]
    fn test(&self) -> StencilTest { self.test }
}

/// Generic stencil buffer attachment for any `StencilType`
#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq)]
pub struct GenericStencil<T: StencilType>(T);

impl<T> Stencil for GenericStencil<T> where T: StencilType {
    type Type = T;
    type Config = GenericStencilConfig<T>;
}