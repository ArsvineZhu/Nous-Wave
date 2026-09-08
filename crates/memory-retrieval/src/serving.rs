use crate::*;
use arc_swap::ArcSwap;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone)]
pub struct ServingSnapshot {
    pub generation: u64,
    pub lexical: Option<Arc<LexicalGeneration>>,
    pub dense: Vec<Arc<DenseGeneration>>,
    pub wave: Option<Arc<WaveGraphGeneration>>,
    pub epa: Vec<Arc<EpaBasisGeneration>>,
    pub postings: Arc<ExactPostings>,
    pub postings_generation: Option<ServingGenerationId>,
}

impl Default for ServingSnapshot {
    fn default() -> Self {
        Self {
            generation: 0,
            lexical: None,
            dense: Vec::new(),
            wave: None,
            epa: Vec::new(),
            postings: Arc::new(ExactPostings::default()),
            postings_generation: None,
        }
    }
}

#[derive(Clone)]
pub struct ServingPublisher {
    current: Arc<ArcSwap<BTreeMap<nous_core::SubjectId, Arc<ServingSnapshot>>>>,
    next_generation: Arc<AtomicU64>,
}

impl Default for ServingPublisher {
    fn default() -> Self {
        Self {
            current: Arc::new(ArcSwap::from_pointee(BTreeMap::new())),
            next_generation: Arc::new(AtomicU64::new(1)),
        }
    }
}

impl ServingPublisher {
    pub fn update_for(&self, subject: nous_core::SubjectId, update: impl Fn(&mut ServingSnapshot)) {
        let generation = self.next_generation();
        self.current.rcu(|current| {
            let mut next = current.as_ref().clone();
            let mut snapshot = next.get(&subject).map(|snapshot| snapshot.as_ref().clone()).unwrap_or_default();
            update(&mut snapshot);
            snapshot.generation = generation;
            next.insert(subject, Arc::new(snapshot));
            Arc::new(next)
        });
    }
    /// Allocate a process-local monotonic generation number.  The database
    /// generation row remains the durable identity; this number is only the
    /// in-process publication trace.
    pub fn next_generation(&self) -> u64 {
        self.next_generation.fetch_add(1, Ordering::Relaxed)
    }

    pub fn snapshot_for(&self, subject: nous_core::SubjectId) -> Arc<ServingSnapshot> {
        self.current
            .load()
            .get(&subject)
            .cloned()
            .unwrap_or_else(|| Arc::new(ServingSnapshot::default()))
    }

    pub fn snapshot(&self) -> Arc<ServingSnapshot> {
        self.current
            .load()
            .values()
            .max_by_key(|snapshot| snapshot.generation)
            .cloned()
            .unwrap_or_else(|| Arc::new(ServingSnapshot::default()))
    }

    pub fn publish_for(&self, subject: nous_core::SubjectId, snapshot: ServingSnapshot) {
        let snapshot = Arc::new(snapshot);
        self.current.rcu(|current| {
            let mut next = current.as_ref().clone();
            next.insert(subject, snapshot.clone());
            Arc::new(next)
        });
    }
}
