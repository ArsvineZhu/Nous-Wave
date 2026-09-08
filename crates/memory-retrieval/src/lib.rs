//! Read-only serving adapters, bounded retrieval and evidence-aware ranking.
use nous_core::{CognitiveRef, Error, EvidenceFamily, Result, ServingGenerationId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
pub type SparseField = BTreeMap<u32, f64>;

pub mod dense;
pub mod lexical;
pub mod residual;
mod graph;
mod wave;
mod fields;
mod trace;
mod ranking;
mod cue_sensing;
mod serving;
mod exact;

pub use dense::{DenseGeneration, DenseMatch, VectorRecord};
pub use lexical::{LexicalDocument, LexicalGeneration, LexicalMatch};
pub use residual::*;
pub use graph::*;
pub use wave::*;
pub use fields::*;
pub use trace::*;
pub use ranking::*;
pub use cue_sensing::*;
pub use serving::*;
pub use exact::*;

#[cfg(test)]
mod tests;
