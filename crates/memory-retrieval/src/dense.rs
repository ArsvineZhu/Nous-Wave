use crate::{RetrievalProjection, projection_error};
use arrow_array::{
    Array, FixedSizeListArray, Float32Array, RecordBatch, RecordBatchIterator, StringArray,
    types::Float32Type,
};
use arrow_schema::{DataType, Field, Schema};
use futures::TryStreamExt;
use lancedb::{
    database::CreateTableMode,
    query::{ExecutableQuery, QueryBase},
};
use nous_core::{Error, Result, SubjectId};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct VectorRow {
    pub subject: SubjectId,
    pub object: Uuid,
    pub revision: Uuid,
    pub vector: Vec<f32>,
}

pub fn model_table(
    model_identity: &str,
    revision: &str,
    preprocessing: &str,
    dimension: usize,
) -> String {
    let identity = format!("{model_identity}\0{revision}\0{preprocessing}\0{dimension}");
    format!("vectors_{}", blake3::hash(identity.as_bytes()).to_hex())
}
fn validate_table(table: &str) -> Result<()> {
    if !table.starts_with("vectors_")
        || table.len() != 72
        || !table[8..].bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(Error::Invalid("invalid model projection identity".into()));
    }
    Ok(())
}
fn schema(dimension: usize) -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("subject_id", DataType::Utf8, false),
        Field::new("object_id", DataType::Utf8, false),
        Field::new("revision_id", DataType::Utf8, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                dimension as i32,
            ),
            true,
        ),
    ]))
}

impl RetrievalProjection {
    pub async fn upsert(&self, table_name: &str, rows: &[VectorRow]) -> Result<()> {
        validate_table(table_name)?;
        if rows.is_empty() {
            return Ok(());
        }
        let dimension = rows[0].vector.len();
        let subject = rows[0].subject;
        if dimension == 0
            || dimension > 65536
            || rows.iter().any(|row| {
                row.subject != subject
                    || row.vector.len() != dimension
                    || row.vector.iter().any(|v| !v.is_finite())
            })
        {
            return Err(Error::Invalid(
                "embedding dimensions or values are invalid".into(),
            ));
        }
        let schema = schema(dimension);
        let table = self
            .connection
            .create_empty_table(
                format!("{table_name}_{}", subject.0.simple()),
                schema.clone(),
            )
            .mode(CreateTableMode::exist_ok(|request| request))
            .execute()
            .await
            .map_err(projection_error)?;
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(
                    rows.iter()
                        .map(|row| row.subject.0.to_string())
                        .collect::<Vec<_>>(),
                )),
                Arc::new(StringArray::from(
                    rows.iter()
                        .map(|row| row.object.to_string())
                        .collect::<Vec<_>>(),
                )),
                Arc::new(StringArray::from(
                    rows.iter()
                        .map(|row| row.revision.to_string())
                        .collect::<Vec<_>>(),
                )),
                Arc::new(
                    FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                        rows.iter().map(|row| {
                            Some(row.vector.iter().copied().map(Some).collect::<Vec<_>>())
                        }),
                        dimension as i32,
                    ),
                ),
            ],
        )
        .map_err(|e| Error::Infrastructure(e.to_string()))?;
        let mut merge = table.merge_insert(&["revision_id"]);
        merge
            .when_matched_update_all(None)
            .when_not_matched_insert_all();
        merge
            .execute(Box::new(RecordBatchIterator::new(vec![Ok(batch)], schema)))
            .await
            .map_err(projection_error)?;
        Ok(())
    }

    pub async fn search(
        &self,
        table_name: &str,
        subject: SubjectId,
        vector: &[f32],
        limit: usize,
    ) -> Result<Vec<Uuid>> {
        validate_table(table_name)?;
        if limit == 0 || limit > 10000 || vector.is_empty() || vector.iter().any(|v| !v.is_finite())
        {
            return Err(Error::Invalid("invalid dense search bounds".into()));
        }
        let table = self
            .connection
            .open_table(format!("{table_name}_{}", subject.0.simple()))
            .execute()
            .await
            .map_err(projection_error)?;
        let batches = table
            .query()
            .only_if(format!("subject_id = '{}'", subject.0))
            .limit(limit)
            .nearest_to(vector)
            .map_err(projection_error)?
            .execute()
            .await
            .map_err(projection_error)?
            .try_collect::<Vec<_>>()
            .await
            .map_err(projection_error)?;
        let mut revisions = vec![];
        for batch in batches {
            let ids = batch
                .column_by_name("revision_id")
                .and_then(|column| column.as_any().downcast_ref::<StringArray>())
                .ok_or_else(|| Error::Infrastructure("invalid dense projection schema".into()))?;
            for id in ids.iter().flatten() {
                revisions
                    .push(Uuid::parse_str(id).map_err(|e| Error::Infrastructure(e.to_string()))?);
            }
        }
        Ok(revisions)
    }

    pub async fn search_rows(
        &self,
        table_name: &str,
        subject: SubjectId,
        vector: &[f32],
        limit: usize,
    ) -> Result<Vec<VectorRow>> {
        validate_table(table_name)?;
        if limit == 0 || limit > 10000 || vector.is_empty() || vector.iter().any(|v| !v.is_finite())
        {
            return Err(Error::Invalid("invalid dense search bounds".into()));
        }
        let table = self
            .connection
            .open_table(format!("{table_name}_{}", subject.0.simple()))
            .execute()
            .await
            .map_err(projection_error)?;
        let batches = table
            .query()
            .only_if(format!("subject_id = '{}'", subject.0))
            .limit(limit)
            .nearest_to(vector)
            .map_err(projection_error)?
            .execute()
            .await
            .map_err(projection_error)?
            .try_collect::<Vec<_>>()
            .await
            .map_err(projection_error)?;
        let mut rows = vec![];
        for batch in batches {
            let objects = batch
                .column_by_name("object_id")
                .and_then(|column| column.as_any().downcast_ref::<StringArray>())
                .ok_or_else(|| Error::Infrastructure("invalid dense projection schema".into()))?;
            let revisions = batch
                .column_by_name("revision_id")
                .and_then(|column| column.as_any().downcast_ref::<StringArray>())
                .ok_or_else(|| Error::Infrastructure("invalid dense projection schema".into()))?;
            let vectors = batch
                .column_by_name("vector")
                .and_then(|column| column.as_any().downcast_ref::<FixedSizeListArray>())
                .ok_or_else(|| {
                    Error::Infrastructure("invalid dense projection vector column".into())
                })?;
            for index in 0..batch.num_rows() {
                let values = vectors.value(index);
                let values = values
                    .as_any()
                    .downcast_ref::<Float32Array>()
                    .ok_or_else(|| {
                        Error::Infrastructure("invalid dense projection vector values".into())
                    })?;
                let object = objects
                    .value(index)
                    .parse()
                    .map_err(|e: uuid::Error| Error::Infrastructure(e.to_string()))?;
                let revision = revisions
                    .value(index)
                    .parse()
                    .map_err(|e: uuid::Error| Error::Infrastructure(e.to_string()))?;
                let vector = values.values().to_vec();
                if vector.iter().any(|value| !value.is_finite()) {
                    return Err(Error::Infrastructure("nonfinite retained embedding".into()));
                }
                rows.push(VectorRow {
                    subject,
                    object,
                    revision,
                    vector,
                });
            }
        }
        Ok(rows)
    }

    pub async fn remove_subject(&self, subject: SubjectId) -> Result<()> {
        for name in self
            .connection
            .table_names()
            .execute()
            .await
            .map_err(projection_error)?
        {
            if name.len() == 105
                && name.ends_with(&format!("_{}", subject.0.simple()))
                && validate_table(&name[..72]).is_ok()
            {
                self.connection
                    .drop_table(&name, &[])
                    .await
                    .map_err(projection_error)?;
            }
        }
        Ok(())
    }
}
