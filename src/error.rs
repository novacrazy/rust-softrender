#[derive(Debug, Fail)]
pub enum RenderError {
    #[fail(display = "Indices already exist")]
    IndicesAlreadyExist,
    #[fail(display = "{} vertices is invalid for {:?} primitive indexing", _0, _1)]
    InvalidVertexCount(usize, ::mesh::Primitive),
}

pub type RenderResult<T> = Result<T, RenderError>;
