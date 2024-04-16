use std::collections::BTreeMap;

use crate::{
    error::{ManycoreError, ManycoreErrorKind},
    Core, Directions, ManycoreSystem, WithID, WithXMLAttributes, ID_KEY,
};

static TASK_KEY: &'static str = "@allocated_task";

impl ManycoreSystem {
    /// Wrapper to generate an [`InfoError`][ManycoreErrorKind::InfoError].
    fn info_error(&self, reason: &'static str) -> ManycoreError {
        ManycoreError::new(ManycoreErrorKind::InfoError(reason))
    }

    /// Gets all available info for specific core or router.
    /// group_id looks something like "r1" or "c20", where r (router) and c (core) symbolise the variant,
    /// and the number is the element's index.
    pub fn get_core_router_specific_info(
        &self,
        ref group_id: String,
    ) -> Result<Option<BTreeMap<String, String>>, ManycoreError> {
        if group_id.len() == 0 {
            return Err(self.info_error("Empty group_id."));
        };

        // Derive group individual information parts from group_id
        let group_split = group_id.split("_").collect::<Vec<&str>>();

        let mut variant_chars = group_split[0].chars();
        let variant_char = variant_chars.next().ok_or(
            self.info_error("Something went wrong retrieving this element's information."),
        )?;
        let id_char = variant_chars
            .next()
            .ok_or(self.info_error("Invalid group id."))?;

        let core: &Core = self
            .cores()
            .list()
            .get(
                usize::try_from(
                    id_char
                        .to_digit(10)
                        .ok_or(self.info_error("Invalid group_id."))?,
                )
                .map_err(|_| self.info_error("Invalid group id."))?,
            )
            .ok_or(self.info_error("Invalid index."))?;

        // id and allocated_task are not part of the core "other_attributes" field so we shall
        // add them manually.
        let insert_core_default = |mut tree: BTreeMap<String, String>| {
            tree.insert(ID_KEY.into(), core.id().to_string());

            if let Some(task_id) = core.allocated_task() {
                tree.insert(TASK_KEY.into(), task_id.to_string());
            }

            tree
        };

        match variant_char {
            'r' => {
                // All relevant router info is already stored in the "other_attributes" map.
                let attributes_clone = core.router().other_attributes().clone();

                Ok(attributes_clone)
            }
            'c' => {
                let attributes_clone = core.other_attributes().clone();

                // We clone the core's map and insert missing fields.
                match attributes_clone {
                    Some(attributes) => Ok(Some(insert_core_default(attributes))),
                    None => Ok(Some(insert_core_default(BTreeMap::new()))),
                }
            }
            'l' => {
                let direction: Directions = (*group_split
                    .get(1)
                    .ok_or(self.info_error("Invalid channel ID."))?)
                .try_into()?;

                // All relevant link info is already stored in the "other_attributes" map.
                let attributes_clone = core
                    .channels()
                    .channel()
                    .get(&direction)
                    .ok_or(self.info_error("Channel direction mismatch: Could not retrieve this channel's information."))?
                    .other_attributes()
                    .clone();

                Ok(attributes_clone)
            }
            _ => Err(self.info_error("Invalid variant.")),
        }
    }
}
