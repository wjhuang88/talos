//! Bounded, materialized provenance carried by private fork receipts.
//!
//! These values describe origins only. They grant no authority without a validated
//! private receipt binding the corresponding exact raw prefix.

use crate::SessionError;
use uuid::Uuid;

const MAX_RUNS: usize = 256;
const MAX_RECORDS: usize = 100_000;

#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub(crate) struct OriginRun {
    pub origin: Uuid,
    pub records: usize,
}

#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
pub(crate) struct Origins(Vec<OriginRun>);

fn invalid() -> SessionError {
    SessionError::ParseError("fork origin relation unavailable".into())
}

impl Origins {
    /// Validate an already receipt-bound inherited prefix, never infer ancestry
    /// from overlapping IDs or from caller-provided transcript metadata.
    pub(crate) fn from_runs(runs: Vec<OriginRun>, count: usize) -> Result<Self, SessionError> {
        if count == 0 || count > MAX_RECORDS || runs.is_empty() || runs.len() > MAX_RUNS {
            return Err(invalid());
        }
        let mut total = 0usize;
        let mut previous = None;
        for run in &runs {
            if run.origin.is_nil() || run.records == 0 || previous == Some(run.origin) {
                return Err(invalid());
            }
            total = total.checked_add(run.records).ok_or_else(invalid)?;
            previous = Some(run.origin);
        }
        if total != count {
            return Err(invalid());
        }
        Ok(Self(runs))
    }

    /// Materialize a child's selected prefix. Parent records after its validated
    /// inheritance belong to the parent, not to the parent's own ancestor.
    pub(crate) fn inherit(
        parent: Uuid,
        inherited: Option<&Self>,
        parent_records: usize,
        selected: usize,
    ) -> Result<Self, SessionError> {
        if parent.is_nil()
            || selected == 0
            || selected > parent_records
            || parent_records > MAX_RECORDS
        {
            return Err(invalid());
        }
        let mut result: Vec<OriginRun> = Vec::new();
        let mut remaining = selected;
        if let Some(inherited) = inherited {
            let count = inherited
                .0
                .iter()
                .try_fold(0usize, |sum, run| sum.checked_add(run.records))
                .ok_or_else(invalid)?;
            inherited.validate(count)?;
            if count > parent_records {
                return Err(invalid());
            }
            for run in &inherited.0 {
                if remaining == 0 {
                    break;
                }
                let records = remaining.min(run.records);
                result.push(OriginRun {
                    origin: run.origin,
                    records,
                });
                remaining -= records;
            }
        }
        if remaining > 0 {
            if let Some(last) = result.last_mut().filter(|last| last.origin == parent) {
                last.records += remaining;
            } else {
                result.push(OriginRun {
                    origin: parent,
                    records: remaining,
                });
            }
        }
        Self::from_runs(result, selected)
    }

    pub(crate) fn validate(&self, count: usize) -> Result<(), SessionError> {
        Self::from_runs(self.0.clone(), count).map(|_| ())
    }

    pub(crate) fn at(&self, index: usize) -> Option<Uuid> {
        let mut offset = index;
        for run in &self.0 {
            if offset < run.records {
                return Some(run.origin);
            }
            offset -= run.records;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_and_sibling_prefixes_preserve_parent_suffix_origin() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let b_prefix = Origins::inherit(a, None, 205, 180).expect("A -> B");
        let c_prefix = Origins::inherit(b, Some(&b_prefix), 190, 187).expect("B -> C");
        let sibling = Origins::inherit(a, None, 205, 185).expect("A -> sibling");
        assert_eq!(c_prefix.at(179), Some(a));
        assert_eq!(c_prefix.at(180), Some(b));
        assert_ne!(c_prefix.at(180), sibling.at(180));
        assert_eq!(c_prefix.at(187), None);
        let nested = Origins::inherit(c, Some(&c_prefix), 195, 190).expect("C -> next");
        assert_eq!(nested.at(186), Some(b));
        assert_eq!(nested.at(187), Some(c));
        let early = Origins::inherit(c, Some(&c_prefix), 195, 50).expect("early cutoff");
        assert_eq!(
            early,
            Origins::inherit(a, None, 205, 50).expect("same prefix")
        );
    }

    #[test]
    fn malformed_and_unbounded_relations_refuse() {
        let id = Uuid::new_v4();
        for runs in [
            vec![],
            vec![OriginRun {
                origin: id,
                records: 0,
            }],
            vec![OriginRun {
                origin: Uuid::nil(),
                records: 1,
            }],
            vec![
                OriginRun {
                    origin: id,
                    records: 1
                };
                2
            ],
            vec![OriginRun {
                origin: id,
                records: usize::MAX,
            }],
        ] {
            assert!(Origins::from_runs(runs, 1).is_err());
        }
        assert!(Origins::inherit(id, None, MAX_RECORDS + 1, 1).is_err());
        assert!(Origins::inherit(id, None, 1, 2).is_err());
        let runs = (0..MAX_RUNS)
            .map(|_| OriginRun {
                origin: Uuid::new_v4(),
                records: 1,
            })
            .collect();
        let prefix = Origins::from_runs(runs, MAX_RUNS).expect("limit");
        assert!(Origins::inherit(id, Some(&prefix), MAX_RUNS + 1, MAX_RUNS + 1).is_err());
        assert!(Origins::inherit(id, Some(&prefix), MAX_RUNS - 1, 1).is_err());
        let malformed = Origins(vec![
            OriginRun {
                origin: id,
                records: usize::MAX,
            },
            OriginRun {
                origin: Uuid::new_v4(),
                records: 1,
            },
        ]);
        assert!(Origins::inherit(id, Some(&malformed), 10, 1).is_err());
    }
}
