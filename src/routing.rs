use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    error::ManycoreError, BTreeVectorKeys, Borders, Core, Cores, Directions, Edge, ManycoreSystem,
    SinkSourceDirection, WithXMLAttributes,
};

/// An enum storing all supported routing algorithms.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum RoutingAlgorithms {
    Observed,
    RowFirst,
    ColumnFirst,
}

/// Array used to expose supported algorithms as a configurable field.
pub static SUPPORTED_ALGORITHMS: [RoutingAlgorithms; 3] = [
    RoutingAlgorithms::Observed,
    RoutingAlgorithms::RowFirst,
    RoutingAlgorithms::ColumnFirst,
];

/// Provides information for routing a task graph edge.
struct EdgeRoutingInformation {
    /// The source core id.
    start_id: u8,
    /// The source core column.
    start_column: u8,
    /// The destination core id.
    destination_id: u8,
    /// The current routing column.
    current_column: u8,
    /// The current routing row.
    current_row: u8,
    /// The destination core column.
    destination_column: u8,
    /// The destination core row.
    destination_row: u8,
    /// The edge cost.
    communication_cost: u16,
    /// The source direction, if any.
    source_direction: Option<SinkSourceDirection>,
    /// The sink direction, if any.
    sink_direction: Option<SinkSourceDirection>,
}

#[derive(Eq, Hash, PartialEq, Clone)]
pub enum RoutingTarget {
    Core(usize),
    Sink(usize),
    Source(usize),
}

pub fn routing_error(reason: String) -> ManycoreError {
    ManycoreError::new(crate::error::ManycoreErrorKind::RoutingError(reason))
}

fn no_core(i: &usize) -> ManycoreError {
    routing_error(format!("Could not get a core with ID {}.", i))
}

pub fn get_core(cores: &mut Cores, i: usize) -> Result<&mut Core, ManycoreError> {
    cores.list_mut().get_mut(i).ok_or(no_core(&i))
}

fn handle_borders(
    cores: &mut Cores,
    add_to_ret: &mut impl FnMut(RoutingTarget, Directions),
    eri: &EdgeRoutingInformation,
) -> Result<(), ManycoreError> {
    if let Some(source_direction) = eri.source_direction.as_ref() {
        let direction = source_direction.into();
        let start_idx = usize::from(eri.start_id);

        add_to_ret(RoutingTarget::Source(start_idx), direction);
    }

    if let Some(sink_direction) = eri.sink_direction.as_ref() {
        let direction = sink_direction.into();
        let destination_idx = usize::from(eri.destination_id);

        add_to_ret(RoutingTarget::Sink(destination_idx), direction);

        get_core(cores, destination_idx)?
            .channels_mut()
            .add_to_cost(eri.communication_cost, direction)?;
    }

    Ok(())
}

impl ManycoreSystem {
    /// Returns the core upon which the given task id is mapped.
    fn task_id_to_core<'a>(
        task_core_map: &HashMap<u16, usize>,
        task_id: u16,
        borders: &Borders,
        cores: &'a Cores,
    ) -> Result<(&'a Core, Option<SinkSourceDirection>), ManycoreError> {
        match task_core_map.get(&task_id) {
            Some(i) => Ok((cores.list().get(*i).ok_or(no_core(i))?, None)),
            None => {
                let key = BTreeVectorKeys::u16(task_id);
                if let Some(sink) = borders.sinks().get(&key) {
                    let idx = sink.core_id();

                    Ok((
                        cores.list().get(*idx).ok_or(no_core(idx))?,
                        Some(sink.direction().clone()),
                    ))
                } else if let Some(source) = borders.sources().get(&key) {
                    let idx = source.core_id();

                    Ok((
                        cores.list().get(*idx).ok_or(no_core(idx))?,
                        Some(source.direction().clone()),
                    ))
                } else {
                    Err(routing_error(format!("Malformed TaskGraph: Task {} is not allocated on any core, sink or source.", task_id)))
                }
            }
        }
    }

    /// Calculates required routing information for the given task graph edge.
    fn calculate_edge_routing_information(
        cores: &Cores,
        borders: &Borders,
        task_core_map: &HashMap<u16, usize>,
        edge: &Edge,
        columns: &u8,
        rows: &u8,
    ) -> Result<EdgeRoutingInformation, ManycoreError> {
        let (start, source) =
            ManycoreSystem::task_id_to_core(task_core_map, *edge.from(), borders, cores)?;

        let (destination, sink) =
            ManycoreSystem::task_id_to_core(task_core_map, *edge.to(), borders, cores)?;

        let start_id = *start.id();
        let destination_id = *destination.id();

        let current_column = start_id % columns;
        let start_column = current_column.clone();
        let current_row = start_id / rows;
        let destination_column = destination_id % columns;
        let destination_row = destination_id / rows;

        Ok(EdgeRoutingInformation {
            start_id,
            start_column,
            destination_id,
            current_column,
            current_row,
            destination_column,
            destination_row,
            communication_cost: *edge.communication_cost(),
            source_direction: source,
            sink_direction: sink,
        })
    }

    /// RowFirst algorithm implementation.
    fn row_first(&mut self) -> Result<HashMap<RoutingTarget, Vec<Directions>>, ManycoreError> {
        let ManycoreSystem {
            ref mut cores,
            ref columns,
            ref rows,
            ref task_graph,
            ref borders,
            ref task_core_map,
            ..
        } = *self;

        // Return value. Stores non-zero core-edge pairs.
        let mut ret: HashMap<RoutingTarget, Vec<Directions>> = HashMap::new();
        // This closure adds a key-value pair to the result.
        let mut add_to_ret = |key: RoutingTarget, direction: Directions| {
            ret.entry(key).or_insert(Vec::new()).push(direction);
        };

        // For each edge in the task graph
        for edge in task_graph.edges() {
            let mut eri = ManycoreSystem::calculate_edge_routing_information(
                cores,
                borders,
                task_core_map,
                edge,
                columns,
                rows,
            )?;

            handle_borders(cores, &mut add_to_ret, &eri)?;

            let mut current_idx = usize::from(eri.start_id);
            let mut core;

            // We must update every connection in the routers matrix
            loop {
                core = get_core(cores, current_idx)?;

                let ret_key = RoutingTarget::Core(current_idx);

                let channels = core.channels_mut();

                if eri.destination_row != eri.current_row {
                    // Row first
                    if eri.start_id > eri.destination_id {
                        // Going up
                        add_to_ret(ret_key, Directions::North);

                        let _ = channels.add_to_cost(eri.communication_cost, Directions::North);
                        current_idx -= usize::from(*columns);
                        eri.current_row -= 1;
                    } else {
                        // Going down
                        add_to_ret(ret_key, Directions::South);

                        let _ = channels.add_to_cost(eri.communication_cost, Directions::South);
                        current_idx += usize::from(*columns);
                        eri.current_row += 1;
                    }
                } else if eri.destination_column != eri.current_column {
                    // Then column
                    if eri.start_column > eri.destination_column {
                        // Going left
                        add_to_ret(ret_key, Directions::West);

                        let _ = channels.add_to_cost(eri.communication_cost, Directions::West);
                        current_idx -= 1;
                        eri.current_column -= 1;
                    } else {
                        // Going right
                        add_to_ret(ret_key, Directions::East);

                        let _ = channels.add_to_cost(eri.communication_cost, Directions::East);
                        current_idx += 1;
                        eri.current_column += 1;
                    }
                } else {
                    // We reached the destination
                    break;
                }
            }
        }

        Ok(ret)
    }

    /// ColumnFirst algorithm implementation.
    fn column_first(&mut self) -> Result<HashMap<RoutingTarget, Vec<Directions>>, ManycoreError> {
        let ManycoreSystem {
            ref mut cores,
            ref columns,
            ref rows,
            ref task_graph,
            ref borders,
            ref task_core_map,
            ..
        } = *self;

        // Return value. Stores non-zero core-edge pairs.
        let mut ret: HashMap<RoutingTarget, Vec<Directions>> = HashMap::new();
        // This closure adds a key-value pair to the result.
        let mut add_to_ret = |key: RoutingTarget, direction: Directions| {
            ret.entry(key).or_insert(Vec::new()).push(direction);
        };

        // For each edge in the task graph
        for edge in task_graph.edges() {
            let mut eri = ManycoreSystem::calculate_edge_routing_information(
                cores,
                borders,
                task_core_map,
                edge,
                columns,
                rows,
            )?;

            handle_borders(cores, &mut add_to_ret, &eri)?;

            let mut current_idx = usize::from(eri.start_id);
            let mut core;

            // We must update every connection in the routers matrix
            loop {
                core = get_core(cores, current_idx)?;

                let ret_key = RoutingTarget::Core(current_idx);

                let channels = core.channels_mut();

                if eri.destination_column != eri.current_column {
                    // Column first
                    if eri.start_column > eri.destination_column {
                        // Going left
                        add_to_ret(ret_key, Directions::West);

                        let _ = channels.add_to_cost(eri.communication_cost, Directions::West);
                        current_idx -= 1;
                        eri.current_column -= 1;
                    } else {
                        // Going right
                        add_to_ret(ret_key, Directions::East);

                        let _ = channels.add_to_cost(eri.communication_cost, Directions::East);
                        current_idx += 1;
                        eri.current_column += 1;
                    }
                } else if eri.destination_row != eri.current_row {
                    // Then row

                    if eri.start_id > eri.destination_id {
                        // Going up
                        add_to_ret(ret_key, Directions::North);

                        let _ = channels.add_to_cost(eri.communication_cost, Directions::North);
                        current_idx -= usize::from(*columns);
                        eri.current_row -= 1;
                    } else {
                        // Going down
                        add_to_ret(ret_key, Directions::South);

                        let _ = channels.add_to_cost(eri.communication_cost, Directions::South);
                        current_idx += usize::from(*columns);
                        eri.current_row += 1;
                    }
                } else {
                    // We reached the destination
                    break;
                }
            }
        }

        Ok(ret)
    }

    /// Observed route implementation. Mirrors Channels information.
    fn observed_route(&mut self) -> Result<HashMap<RoutingTarget, Vec<Directions>>, ManycoreError> {
        let cores = self.cores_mut();

        let mut ret: HashMap<RoutingTarget, Vec<Directions>> = HashMap::new();

        let mut add_to_ret = |key: RoutingTarget, direction: Directions| {
            ret.entry(key).or_insert(Vec::new()).push(direction);
        };

        let mut core;
        for i in 0..cores.list().len() {
            let ret_key = RoutingTarget::Core(i);
            core = get_core(cores, i)?;
            for (direction, channel) in core.channels_mut().channel_mut() {
                let packets = *channel.packets_transmitted();
                if packets != 0 {
                    add_to_ret(ret_key.clone(), *direction);

                    channel.add_to_cost(packets);
                }
            }
        }

        Ok(ret)
    }

    /// Clears all links loads.
    fn clear_links(&mut self) {
        // Zero out all links costs
        self.cores_mut()
            .list_mut()
            .iter_mut()
            .for_each(|c| c.channels_mut().clear_loads());
    }

    /// Performs routing according to the requested algorithm.
    pub fn route(
        &mut self,
        algorithm: &RoutingAlgorithms,
    ) -> Result<HashMap<RoutingTarget, Vec<Directions>>, ManycoreError> {
        self.clear_links();

        match algorithm {
            RoutingAlgorithms::ColumnFirst => self.column_first(),
            RoutingAlgorithms::RowFirst => self.row_first(),
            RoutingAlgorithms::Observed => self.observed_route(),
        }
    }
}
