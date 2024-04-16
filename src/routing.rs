use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::{Deserialize, Serialize};

use crate::{
    error::ManycoreError, Borders, Core, Cores, Directions, Edge, ManycoreErrorKind,
    ManycoreSystem, SinkSourceDirection, WithID,
};

/// An enum storing all supported routing algorithms.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum RoutingAlgorithms {
    Observed,
    RowFirst,
    ColumnFirst,
}

/// Array used to expose supported algorithms as a configurable field.
pub(crate) static SUPPORTED_ALGORITHMS: [RoutingAlgorithms; 3] = [
    RoutingAlgorithms::Observed,
    RoutingAlgorithms::RowFirst,
    RoutingAlgorithms::ColumnFirst,
];

#[derive(Debug)]
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

/// Enum to differentiate targets of routing packets.
#[derive(Eq, Hash, PartialEq, Clone, Debug, PartialOrd, Ord)]
pub enum RoutingTarget {
    CoreSink,
    Source,
}

/// Wapper function to generate [`ManycoreErrorKind::RoutingError`].
pub(crate) fn routing_error(reason: String) -> ManycoreError {
    ManycoreError::new(ManycoreErrorKind::RoutingError(reason))
}

/// Wrapper function to generate a [`ManycoreErrorKind::RoutingError`] caused by not finding a [`Core`] having the requested ID.
fn no_core(i: &usize) -> ManycoreError {
    routing_error(format!("Could not get a core with ID {}.", i))
}
/// Wrapper function to generate a [`ManycoreErrorKind::RoutingError`] caused by not finding a [`Task`] having the requested ID.
fn no_task(i: &u16) -> ManycoreError {
    routing_error(format!(
        "Malformed TaskGraph: Task {} is not allocated on any core, sink or source.",
        i
    ))
}

/// Utility function to retrieve a core by ID. Wraps in [`Result`] for convenience.
pub(crate) fn get_core(cores: &mut Cores, i: usize) -> Result<&mut Core, ManycoreError> {
    cores.list_mut().get_mut(i).ok_or(no_core(&i))
}

/// Type of a successfully genereated routing result map.
pub type RoutingMap = HashMap<u8, BTreeMap<RoutingTarget, BTreeSet<Directions>>>;

/// Utility function to add routing data to the routing result map.
fn add_to_ret(key: u8, target: RoutingTarget, direction: Directions, ret: &mut RoutingMap) {
    ret.entry(key)
        .or_insert(BTreeMap::default())
        .entry(target)
        .or_insert(BTreeSet::default())
        .insert(direction);
}

/// Utility function to add borders routing information to the routing result map.
fn handle_borders(
    cores: &mut Cores,
    ret: &mut RoutingMap,
    eri: &EdgeRoutingInformation,
) -> Result<(), ManycoreError> {
    // Was the task graph edge routed through a source?
    if let Some(source_direction) = eri.source_direction.as_ref() {
        // If so, we'll want to display load of the source channel. Add to map.
        let direction = source_direction.into();
        let start_idx = usize::from(eri.start_id);

        add_to_ret(eri.start_id, RoutingTarget::Source, direction, ret);

        // Output connections from sources are not part of the input XML.
        // We must cumulatively track the load here.
        get_core(cores, start_idx)?.add_source_load(eri.communication_cost, &direction)?;
    }

    // Was the task graph edge rrouted through a sink?
    if let Some(sink_direction) = eri.sink_direction.as_ref() {
        // If so, we'll want to display load of the sink channel. Add to map.
        let direction = sink_direction.into();
        let destination_idx = usize::from(eri.destination_id);

        add_to_ret(eri.destination_id, RoutingTarget::CoreSink, direction, ret);

        // A sink incoming link is actually a core's outgoing channel.
        // Cumulatively track the load on the channel.
        // We do it here because sinks are not actually part of the inner
        // algorithmically routable connections matrix.
        // The routing algorithm will stop upon reaching the target core (column, row) pair.
        get_core(cores, destination_idx)?
            .channels_mut()
            .add_to_load(eri.communication_cost, direction)?;
    }

    Ok(())
}

impl ManycoreSystem {
    /// Returns the core upon which the given task id is mapped.
    fn task_id_to_core<'a>(
        task_core_map: &HashMap<u16, usize>,
        task_id: u16,
        borders: &mut Option<Borders>,
        cores: &'a Cores,
    ) -> Result<(&'a Core, Option<SinkSourceDirection>), ManycoreError> {
        match task_core_map.get(&task_id) {
            // Lucky base case, the task is allocated on a core.
            Some(i) => Ok((cores.list().get(*i).ok_or(no_core(i))?, None)),
            None => match borders {
                // The task is hopefuly coming from a source or is allocated on a sink.
                Some(borders) => {
                    if let Some(sink) = borders.sinks().get(&task_id) {
                        // Task is on a sink
                        let idx = sink.core_id();

                        Ok((
                            cores.list().get(*idx).ok_or(no_core(idx))?,
                            Some(sink.direction().clone()),
                        ))
                    } else if let Some(source) = borders.sources_mut().get_mut(&task_id) {
                        // Task is on a source
                        let idx = *source.core_id();

                        Ok((
                            cores.list().get(idx).ok_or(no_core(&idx))?,
                            Some(source.direction().clone()),
                        ))
                    } else {
                        // The requested task is nowhere to be found. Task graph is invalid.
                        Err(no_task(&task_id))
                    }
                }
                // The requested task is nowhere to be found. Task graph is invalid.
                None => Err(no_task(&task_id)),
            },
        }
    }

    /// Calculates required routing information for the given task graph edge.
    fn calculate_edge_routing_information(
        cores: &Cores,
        borders: &mut Option<Borders>,
        task_core_map: &HashMap<u16, usize>,
        edge: &Edge,
        columns: &u8,
        rows: &u8,
    ) -> Result<EdgeRoutingInformation, ManycoreError> {
        // Retrieve core upon which source task is mapped.
        // Will take care of mapping onto core if coming from source.
        let (start, source) =
            ManycoreSystem::task_id_to_core(task_core_map, *edge.from(), borders, cores)?;

        // Retrieve core upon which destination task is mapped.
        // Will take care of mapping onto core if coming from sink.
        let (destination, sink) =
            ManycoreSystem::task_id_to_core(task_core_map, *edge.to(), borders, cores)?;

        let start_id = *start.id();
        let destination_id = *destination.id();

        // Workout where are we and where do we want to go in inner matrix.
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
    fn row_first(&mut self) -> Result<RoutingMap, ManycoreError> {
        let ManycoreSystem {
            ref mut cores,
            ref columns,
            ref rows,
            ref task_graph,
            ref mut borders,
            ref task_core_map,
            ..
        } = *self;

        // Return value. Stores non-zero core-edge pairs.
        let mut ret: RoutingMap = HashMap::new();

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

            handle_borders(cores, &mut ret, &eri)?;

            let mut current_idx = usize::from(eri.start_id);
            let mut core;

            // We must update every connection in the routers matrix
            loop {
                core = get_core(cores, current_idx)?;
                let core_id = *core.id();

                let channels = core.channels_mut();

                if eri.destination_row != eri.current_row {
                    // Row first
                    if eri.start_id > eri.destination_id {
                        // Going up
                        add_to_ret(
                            core_id,
                            RoutingTarget::CoreSink,
                            Directions::North,
                            &mut ret,
                        );

                        let _ = channels.add_to_load(eri.communication_cost, Directions::North)?;
                        current_idx -= usize::from(*columns);
                        eri.current_row -= 1;
                    } else {
                        // Going down
                        add_to_ret(
                            core_id,
                            RoutingTarget::CoreSink,
                            Directions::South,
                            &mut ret,
                        );

                        let _ = channels.add_to_load(eri.communication_cost, Directions::South)?;
                        current_idx += usize::from(*columns);
                        eri.current_row += 1;
                    }
                } else if eri.destination_column != eri.current_column {
                    // Then column
                    if eri.start_column > eri.destination_column {
                        // Going left
                        add_to_ret(core_id, RoutingTarget::CoreSink, Directions::West, &mut ret);

                        let _ = channels.add_to_load(eri.communication_cost, Directions::West)?;
                        current_idx -= 1;
                        eri.current_column -= 1;
                    } else {
                        // Going right
                        add_to_ret(core_id, RoutingTarget::CoreSink, Directions::East, &mut ret);

                        let _ = channels.add_to_load(eri.communication_cost, Directions::East)?;
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
    fn column_first(&mut self) -> Result<RoutingMap, ManycoreError> {
        let ManycoreSystem {
            ref mut cores,
            ref columns,
            ref rows,
            ref task_graph,
            ref mut borders,
            ref task_core_map,
            ..
        } = *self;

        // Return value. Stores non-zero core-edge pairs.
        let mut ret: RoutingMap = HashMap::new();

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

            handle_borders(cores, &mut ret, &eri)?;

            let mut current_idx = usize::from(eri.start_id);
            let mut core;

            // We must update every connection in the routers matrix
            loop {
                core = get_core(cores, current_idx)?;
                let core_id = *core.id();

                let channels = core.channels_mut();

                if eri.destination_column != eri.current_column {
                    // Column first
                    if eri.start_column > eri.destination_column {
                        // Going left
                        add_to_ret(core_id, RoutingTarget::CoreSink, Directions::West, &mut ret);

                        let _ = channels.add_to_load(eri.communication_cost, Directions::West)?;
                        current_idx -= 1;
                        eri.current_column -= 1;
                    } else {
                        // Going right
                        add_to_ret(core_id, RoutingTarget::CoreSink, Directions::East, &mut ret);

                        let _ = channels.add_to_load(eri.communication_cost, Directions::East)?;
                        current_idx += 1;
                        eri.current_column += 1;
                    }
                } else if eri.destination_row != eri.current_row {
                    // Then row

                    if eri.start_id > eri.destination_id {
                        // Going up
                        add_to_ret(
                            core_id,
                            RoutingTarget::CoreSink,
                            Directions::North,
                            &mut ret,
                        );

                        let _ = channels.add_to_load(eri.communication_cost, Directions::North)?;
                        current_idx -= usize::from(*columns);
                        eri.current_row -= 1;
                    } else {
                        // Going down
                        add_to_ret(
                            core_id,
                            RoutingTarget::CoreSink,
                            Directions::South,
                            &mut ret,
                        );

                        let _ = channels.add_to_load(eri.communication_cost, Directions::South)?;
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
    fn observed_route(&mut self) -> Result<RoutingMap, ManycoreError> {
        let ManycoreSystem {
            ref mut cores,
            ref task_graph,
            ref mut borders,
            ..
        } = *self;

        let mut ret: RoutingMap = HashMap::new();

        let mut core;
        // Copy all core loads over
        for i in 0..cores.list().len() {
            core = get_core(cores, i)?;
            let core_id = *core.id();
            for (direction, channel) in core.channels_mut().channel_mut() {
                let packets = *channel.actual_com_cost();
                if packets != 0 {
                    add_to_ret(core_id, RoutingTarget::CoreSink, *direction, &mut ret);

                    channel.add_to_load(packets);
                }
            }
        }

        // Copy all source loads over
        if let Some(borders) = borders {
            for e in task_graph.edges() {
                if let Some(source) = borders.sources_mut().get_mut(e.from()) {
                    let direction = Directions::from(source.direction());

                    let core = get_core(cores, *source.core_id())?;
                    core.add_source_load(*e.communication_cost(), &direction)?;

                    add_to_ret(*core.id(), RoutingTarget::Source, direction, &mut ret);
                }
            }
        }

        Ok(ret)
    }

    /// Clears all channel loads.
    fn clear_channels(&mut self) {
        // Zero out all links costs
        self.cores_mut().list_mut().iter_mut().for_each(|c| {
            // Channel loads
            c.channels_mut().clear_loads();
            // Source loads
            c.clear_source_loads();
        });
    }

    /// Performs routing according to the requested algorithm.
    pub fn route(&mut self, algorithm: &RoutingAlgorithms) -> Result<RoutingMap, ManycoreError> {
        self.clear_channels();

        match algorithm {
            RoutingAlgorithms::ColumnFirst => self.column_first(),
            RoutingAlgorithms::RowFirst => self.row_first(),
            RoutingAlgorithms::Observed => self.observed_route(),
        }
    }
}
