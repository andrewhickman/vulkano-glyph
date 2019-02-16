use std::{error, fmt, result};

use rusttype::gpu_cache::CacheReadErr;
use vulkano::command_buffer::{
    BuildError, CommandBufferExecError, CopyBufferImageError, DrawIndirectError,
};
use vulkano::descriptor::descriptor_set::{
    PersistentDescriptorSetBuildError, PersistentDescriptorSetError,
};
use vulkano::image::ImageCreationError;
use vulkano::memory::DeviceMemoryAllocError;
use vulkano::pipeline::GraphicsPipelineCreationError;
use vulkano::sampler::SamplerCreationError;
use vulkano::OomError;

/// A type alias for Result<T, vulkano_glyph::Error>.
pub type Result<T> = result::Result<T, Error>;

/// An error that can occur when drawing text.
#[derive(Debug)]
pub struct Error(Box<ErrorKind>);

impl Error {
    fn new(kind: impl Into<Box<ErrorKind>>) -> Self {
        Error(kind.into())
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.0
    }

    pub fn into_kind(self) -> ErrorKind {
        *self.0
    }
}

/// The specific kind of an `Error`.
#[derive(Debug)]
pub enum ErrorKind {
    /// A requested glyph was not in the cache.
    CacheRead(CacheReadErr),
    Build(BuildError),
    CopyBufferImage(CopyBufferImageError),
    CommandBufferExec(CommandBufferExecError),
    DrawIndirect(DrawIndirectError),
    DeviceMemoryAlloc(DeviceMemoryAllocError),
    SamplerCreation(SamplerCreationError),
    ImageCreation(ImageCreationError),
    GraphicsPipelineCreation(GraphicsPipelineCreationError),
    PersistentDescriptorSet(PersistentDescriptorSetError),
    PersistentDescriptorSetBuild(PersistentDescriptorSetBuildError),
    Oom(OomError),
    #[doc(hidden)]
    __NonExhaustive,
}

impl From<CacheReadErr> for Error {
    fn from(err: CacheReadErr) -> Self {
        Error::new(ErrorKind::CacheRead(err))
    }
}

impl From<CopyBufferImageError> for Error {
    fn from(err: CopyBufferImageError) -> Self {
        Error::new(ErrorKind::CopyBufferImage(err))
    }
}

impl From<CommandBufferExecError> for Error {
    fn from(err: CommandBufferExecError) -> Self {
        Error::new(ErrorKind::CommandBufferExec(err))
    }
}

impl From<BuildError> for Error {
    fn from(err: BuildError) -> Self {
        Error::new(ErrorKind::Build(err))
    }
}

impl From<DeviceMemoryAllocError> for Error {
    fn from(err: DeviceMemoryAllocError) -> Self {
        Error::new(ErrorKind::DeviceMemoryAlloc(err))
    }
}

impl From<OomError> for Error {
    fn from(err: OomError) -> Self {
        Error::new(ErrorKind::Oom(err))
    }
}

impl From<SamplerCreationError> for Error {
    fn from(err: SamplerCreationError) -> Self {
        Error::new(ErrorKind::SamplerCreation(err))
    }
}

impl From<ImageCreationError> for Error {
    fn from(err: ImageCreationError) -> Self {
        Error::new(ErrorKind::ImageCreation(err))
    }
}

impl From<GraphicsPipelineCreationError> for Error {
    fn from(err: GraphicsPipelineCreationError) -> Self {
        Error::new(ErrorKind::GraphicsPipelineCreation(err))
    }
}

impl From<DrawIndirectError> for Error {
    fn from(err: DrawIndirectError) -> Self {
        Error::new(ErrorKind::DrawIndirect(err))
    }
}

impl From<PersistentDescriptorSetError> for Error {
    fn from(err: PersistentDescriptorSetError) -> Self {
        Error::new(ErrorKind::PersistentDescriptorSet(err))
    }
}

impl From<PersistentDescriptorSetBuildError> for Error {
    fn from(err: PersistentDescriptorSetBuildError) -> Self {
        Error::new(ErrorKind::PersistentDescriptorSetBuild(err))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind() {
            ErrorKind::CacheRead(err) => err.fmt(f),
            ErrorKind::CopyBufferImage(err) => err.fmt(f),
            ErrorKind::Build(err) => err.fmt(f),
            ErrorKind::CommandBufferExec(err) => err.fmt(f),
            ErrorKind::DrawIndirect(err) => err.fmt(f),
            ErrorKind::DeviceMemoryAlloc(err) => err.fmt(f),
            ErrorKind::SamplerCreation(err) => err.fmt(f),
            ErrorKind::ImageCreation(err) => err.fmt(f),
            ErrorKind::GraphicsPipelineCreation(err) => err.fmt(f),
            ErrorKind::Oom(err) => err.fmt(f),
            ErrorKind::PersistentDescriptorSet(err) => err.fmt(f),
            ErrorKind::PersistentDescriptorSetBuild(err) => err.fmt(f),
            ErrorKind::__NonExhaustive => unreachable!(),
        }
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&dyn error::Error> {
        Some(match self.kind() {
            ErrorKind::CacheRead(err) => err,
            ErrorKind::CopyBufferImage(err) => err,
            ErrorKind::Build(err) => err,
            ErrorKind::CommandBufferExec(err) => err,
            ErrorKind::DrawIndirect(err) => err,
            ErrorKind::DeviceMemoryAlloc(err) => err,
            ErrorKind::SamplerCreation(err) => err,
            ErrorKind::ImageCreation(err) => err,
            ErrorKind::GraphicsPipelineCreation(err) => err,
            ErrorKind::Oom(err) => err,
            ErrorKind::PersistentDescriptorSet(err) => err,
            ErrorKind::PersistentDescriptorSetBuild(err) => err,
            ErrorKind::__NonExhaustive => unreachable!(),
        })
    }
}
