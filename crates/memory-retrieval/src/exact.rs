use crate::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExactPostings {
    postings: HashMap<String, roaring::RoaringBitmap>,
    references: HashMap<u32, Vec<CognitiveRef>>,
}

impl ExactPostings {
    pub fn insert(&mut self, key: impl Into<String>, serving_id: u32) {
        self.postings
            .entry(key.into())
            .or_default()
            .insert(serving_id);
    }

    pub fn insert_reference(
        &mut self,
        key: impl Into<String>,
        serving_id: u32,
        reference: CognitiveRef,
    ) {
        self.insert(key, serving_id);
        let references = self.references.entry(serving_id).or_default();
        if !references.contains(&reference) {
            references.push(reference);
        }
    }

    pub fn get(&self, key: &str) -> Option<&roaring::RoaringBitmap> {
        self.postings.get(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.postings.keys()
    }

    pub fn references(&self, key: &str) -> Vec<CognitiveRef> {
        self.get(key)
            .into_iter()
            .flat_map(|bitmap| bitmap.iter())
            .flat_map(|serving_id| {
                self.references
                    .get(&serving_id)
                    .cloned()
                    .unwrap_or_default()
            })
            .collect()
    }
}
