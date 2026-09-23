//! Private protocol adapters. Domain owners remain the semantic boundary.
mod authority;
mod convert;
mod evidence;
mod identity;
mod management;
mod material;
mod memory;
mod consolidation;
mod model;
mod query;
mod runtime;
mod subject;
mod topology;

use crate::NousRuntime;
use convert::*;
use nous_core::Error;
use nous_protocol::{kernel as k, public as p};
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct KernelService(pub NousRuntime);

pub fn status(error: Error) -> Status {
    match error {
        Error::Invalid(message) => Status::invalid_argument(message),
        Error::NotFound(message) => Status::not_found(message),
        Error::Conflict(message) => Status::aborted(message),
        Error::Unavailable(message) => Status::unavailable(message),
        Error::Infrastructure(message) => {
            tracing::error!(%message, "kernel operation failed");
            Status::internal("Kernel operation failed")
        }
    }
}
