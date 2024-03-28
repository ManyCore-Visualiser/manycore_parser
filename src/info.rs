use std::collections::BTreeMap;

use crate::{
    error::{ManycoreError, ManycoreErrorKind},
    Core, ManycoreSystem, WithXMLAttributes,
};

impl ManycoreSystem {
    fn info_error(&self, reason: &'static str) -> ManycoreError {
        ManycoreError::new(ManycoreErrorKind::InfoError(reason))
    }
    /// Gets all available info for specific core or router.
    /// group_id looks something like "r1" or "c20", where r (router) and c (core) symbolise the variant,
    /// and the number is the element's index.
    pub fn get_core_router_specific_info(
        &self,
        mut group_id: String,
    ) -> Result<Option<BTreeMap<String, String>>, ManycoreError> {
        if group_id.len() == 0 {
            return Err(self.info_error("Empty group_id."));
        };

        let variant_string = group_id.remove(0).to_string();

        let core: &Core = self
            .cores()
            .list()
            .get(
                group_id
                    .parse::<usize>()
                    .map_err(|_| self.info_error("Invalid group_id."))?,
            )
            .ok_or(self.info_error("Invalid index."))?;

        // id and allocated_task are not part of the core "other_attributes" field so we shall
        // add them manually.
        let insert_core_default = |mut tree: BTreeMap<String, String>| {
            tree.insert("@id".into(), core.id().to_string());

            if let Some(task_id) = core.allocated_task() {
                tree.insert("@allocated_task".into(), task_id.to_string());
            }

            tree
        };

        match variant_string.as_str() {
            "r" => {
                // All relevant router info is already stored in the "other_attributes" map.
                let attributes_clone = core.router().other_attributes().clone();

                Ok(attributes_clone)
            }
            "c" => {
                let attributes_clone = core.other_attributes().clone();

                // We clone the core's map and insert missing fields.
                match attributes_clone {
                    Some(attributes) => Ok(Some(insert_core_default(attributes))),
                    None => Ok(Some(insert_core_default(BTreeMap::new()))),
                }
            }
            _ => Err(self.info_error("Invalid variant.")),
        }
    }
}
