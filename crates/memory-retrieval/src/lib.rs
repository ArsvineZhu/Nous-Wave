//! Read-only serving adapters, bounded retrieval and evidence-aware ranking.
use nous_core::{CognitiveRef, Error, EvidenceFamily, Result, ServingGenerationId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
pub type SparseField = BTreeMap<u32, f64>;

mod cue_sensing;
pub mod dense;
mod exact;
mod fields;
mod graph;
pub mod lexical;
mod ranking;
pub mod residual;
mod serving;
mod trace;
mod wave;

pub use cue_sensing::*;
pub use dense::{DenseGeneration, DenseMatch, VectorRecord};
pub use exact::*;
pub use fields::*;
pub use graph::*;
pub use lexical::{LexicalDocument, LexicalGeneration, LexicalMatch};
pub use ranking::*;
pub use residual::*;
pub use serving::*;
pub use trace::*;
pub use wave::*;

#[cfg(test)]
mod tests;
