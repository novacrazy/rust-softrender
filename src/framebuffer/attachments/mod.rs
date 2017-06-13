pub mod color;
pub mod depth;
pub mod stencil;

pub trait Attachment: Copy + Default + Send + Sync + 'static {}

impl<T> Attachment for T where T: Copy + Default + Send + Sync + 'static {}

pub use self::color::Color;
pub use self::depth::Depth;
pub use self::stencil::{Stencil, StencilOp, StencilTest, StencilType, StencilConfig, GenericStencilConfig, GenericStencil};

pub trait Attachments: Attachment {
    type Color: Attachment + Color;
    type Depth: Attachment + Depth;
    type Stencil: Attachment + Stencil;
}

pub mod predefined;

pub use self::predefined::*;