use std::{collections::HashMap, error::Error, fmt::Display};

use getset::{Getters, MutGetters, Setters};
use serde::{Deserialize, Serialize};

use crate::{Cores, Directions, Edge, ManycoreSystem, WithXMLAttributes};

/// A neighbour.
#[derive(Default, Debug, PartialEq, Clone, Getters)]
#[getset(get = "pub")]
pub struct Neighbour {
    /// The neighbour id.
    id: usize,
    /// The load on this channel.
    link_cost: u8,
}

impl Neighbour {
    /// Adds to the channel's load.
    fn add_to_cost(&mut self, cost: u8) {
        self.link_cost += cost;
    }

    /// Instantiates a new neighbour if a neighbour id is given, else returns None.
    pub fn new(id: Option<usize>) -> Option<Neighbour> {
        if let Some(id) = id {
            return Some(Neighbour { id, link_cost: 0 });
        }

        None
    }
}

/// Holds options for each possible neighbour. Is None if there is no connection.
#[derive(Default, Debug, PartialEq, Getters, MutGetters, Setters, Clone)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Neighbours {
    top: Option<Neighbour>,
    right: Option<Neighbour>,
    bottom: Option<Neighbour>,
    left: Option<Neighbour>,
}

impl Neighbours {
    /// Instantiates a new neighbours instance.
    pub fn new(
        top: Option<usize>,
        right: Option<usize>,
        bottom: Option<usize>,
        left: Option<usize>,
    ) -> Self {
        Self {
            top: Neighbour::new(top),
            right: Neighbour::new(right),
            bottom: Neighbour::new(bottom),
            left: Neighbour::new(left),
        }
    }

    /// Resets all loads on every channel.
    fn clear_link_costs(&mut self) {
        if let Some(top) = &mut self.top {
            top.link_cost = 0;
        }

        if let Some(right) = &mut self.right {
            right.link_cost = 0;
        }

        if let Some(bottom) = &mut self.bottom {
            bottom.link_cost = 0;
        }

        if let Some(left) = &mut self.left {
            left.link_cost = 0;
        }
    }
}

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
    communication_cost: u8,
}

#[derive(Debug)]
pub struct ConnectionUpdateError;

impl Display for ConnectionUpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Could not find required neighbour. Task graph might be misshapen."
        )
    }
}

impl Error for ConnectionUpdateError {}

impl ManycoreSystem {
    /// Returns the core upon which the given task id is mapped.
    fn task_id_to_core_id<'a>(
        task_core_map: &'a HashMap<u16, usize>,
        task_id: &'a u16,
    ) -> Result<&'a usize, ConnectionUpdateError> {
        task_core_map.get(task_id).ok_or(ConnectionUpdateError)
    }

    /// Calculates required routing information for the given task graph edge.
    fn calculate_edge_routing_information(
        cores: &Cores,
        task_core_map: &HashMap<u16, usize>,
        edge: &Edge,
        columns: &u8,
        rows: &u8,
    ) -> Result<EdgeRoutingInformation, ConnectionUpdateError> {
        let start = &cores.list()[*ManycoreSystem::task_id_to_core_id(task_core_map, edge.from())?];
        let destination =
            &cores.list()[*ManycoreSystem::task_id_to_core_id(task_core_map, edge.to())?];

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
        })
    }

    /// Updates a neighbour's load.
    fn update_neighbour<'a>(
        neighbours: &'a mut Neighbours,
        cost: u8,
        neighbour_selector: &impl Fn(&mut Neighbours) -> &mut Option<Neighbour>,
    ) -> Result<&'a mut Neighbour, ConnectionUpdateError> {
        let neighbour = neighbour_selector(neighbours)
            .as_mut()
            .ok_or(ConnectionUpdateError)?;

        neighbour.add_to_cost(cost);

        Ok(neighbour)
    }

    /// Updates the current connection and returns the id of the reached neighbour.
    fn update_connection(
        neighbours: &mut Neighbours,
        cost: u8,
        delta_target: &mut u8,
        positive_delta: bool,
        neighbour_selector: &impl Fn(&mut Neighbours) -> &mut Option<Neighbour>,
    ) -> Result<usize, ConnectionUpdateError> {
        let neighbour = ManycoreSystem::update_neighbour(neighbours, cost, neighbour_selector)?;

        // For code reusability, this function can mutate row/column index in both directions.
        if positive_delta {
            *delta_target += 1;
        } else {
            *delta_target -= 1;
        }

        Ok(neighbour.id)
    }

    /// RowFirst algorithm implementation.
    fn row_first(&mut self) -> Result<HashMap<usize, Vec<Directions>>, ConnectionUpdateError> {
        let ManycoreSystem {
            ref cores,
            ref columns,
            ref rows,
            ref task_graph,
            ref mut connections,
            ref task_core_map,
            ..
        } = *self;

        // Return value. Stores non-zero core-edge pairs.
        let mut ret: HashMap<usize, Vec<Directions>> = HashMap::new();

        // This closure adds a key-value pair to the result.
        let mut add_to_ret = |i: usize, direction: Directions| {
            ret.entry(i).or_insert(Vec::new()).push(direction);
        };

        // For each edge in the task graph
        for edge in task_graph.edges() {
            let mut eri = ManycoreSystem::calculate_edge_routing_information(
                cores,
                task_core_map,
                edge,
                columns,
                rows,
            )?;

            let mut current_idx = usize::from(eri.start_id);

            // We must update every connection in the routers matrix
            loop {
                let neighbours = connections
                    .get_mut(&current_idx)
                    .ok_or(ConnectionUpdateError)?;
                if eri.destination_row != eri.current_row {
                    // Row first
                    if eri.start_id > eri.destination_id {
                        // Going up
                        add_to_ret(current_idx, Directions::North);

                        current_idx = ManycoreSystem::update_connection(
                            neighbours,
                            eri.communication_cost,
                            &mut eri.current_row,
                            false,
                            &Neighbours::top_mut,
                        )?;
                    } else {
                        // Going down
                        add_to_ret(current_idx, Directions::South);

                        current_idx = ManycoreSystem::update_connection(
                            neighbours,
                            eri.communication_cost,
                            &mut eri.current_row,
                            true,
                            &Neighbours::bottom_mut,
                        )?;
                    }
                } else if eri.destination_column != eri.current_column {
                    // Then column
                    if eri.start_column > eri.destination_column {
                        // Going left
                        add_to_ret(current_idx, Directions::West);

                        current_idx = ManycoreSystem::update_connection(
                            neighbours,
                            eri.communication_cost,
                            &mut eri.current_column,
                            false,
                            &Neighbours::left_mut,
                        )?;
                    } else {
                        // Going right
                        add_to_ret(current_idx, Directions::East);

                        current_idx = ManycoreSystem::update_connection(
                            neighbours,
                            eri.communication_cost,
                            &mut eri.current_column,
                            true,
                            &Neighbours::right_mut,
                        )?;
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
    fn column_first(&mut self) -> Result<HashMap<usize, Vec<Directions>>, ConnectionUpdateError> {
        let ManycoreSystem {
            ref cores,
            ref columns,
            ref rows,
            ref task_graph,
            ref mut connections,
            ref task_core_map,
            ..
        } = *self;

        // Return value. Stores non-zero core-edge pairs
        let mut ret: HashMap<usize, Vec<Directions>> = HashMap::new();

        // This closure adds a key-value pair to the result.
        let mut add_to_ret = |i: usize, direction: Directions| {
            ret.entry(i).or_insert(Vec::new()).push(direction);
        };

        // For each edge in the task graph
        for edge in task_graph.edges() {
            let mut eri = ManycoreSystem::calculate_edge_routing_information(
                cores,
                task_core_map,
                edge,
                columns,
                rows,
            )?;

            let mut current_idx = usize::from(eri.start_id);

            // We must update every connection in the routers matrix
            loop {
                let neighbours = connections
                    .get_mut(&current_idx)
                    .ok_or(ConnectionUpdateError)?;
                if eri.destination_column != eri.current_column {
                    // Column first
                    if eri.start_column > eri.destination_column {
                        // Going left
                        add_to_ret(current_idx, Directions::West);

                        current_idx = ManycoreSystem::update_connection(
                            neighbours,
                            eri.communication_cost,
                            &mut eri.current_column,
                            false,
                            &Neighbours::left_mut,
                        )?;
                    } else {
                        // Going right
                        add_to_ret(current_idx, Directions::East);

                        current_idx = ManycoreSystem::update_connection(
                            neighbours,
                            eri.communication_cost,
                            &mut eri.current_column,
                            true,
                            &Neighbours::right_mut,
                        )?;
                    }
                } else if eri.destination_row != eri.current_row {
                    // Then row

                    if eri.start_id > eri.destination_id {
                        // Going up
                        add_to_ret(current_idx, Directions::North);

                        current_idx = ManycoreSystem::update_connection(
                            neighbours,
                            eri.communication_cost,
                            &mut eri.current_row,
                            false,
                            &Neighbours::top_mut,
                        )?;
                    } else {
                        // Going down
                        add_to_ret(current_idx, Directions::South);

                        current_idx = ManycoreSystem::update_connection(
                            neighbours,
                            eri.communication_cost,
                            &mut eri.current_row,
                            true,
                            &Neighbours::bottom_mut,
                        )?;
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
    fn observed_route(&mut self) -> Result<HashMap<usize, Vec<Directions>>, ConnectionUpdateError> {
        self.clear_links();
        let ManycoreSystem {
            ref cores,
            ref mut connections,
            ..
        } = *self;

        let mut ret: HashMap<usize, Vec<Directions>> = HashMap::new();

        let mut add_to_ret = |i: usize, direction: Directions| {
            ret.entry(i).or_insert(Vec::new()).push(direction);
        };

        for i in 0..cores.list().len() {
            if let Some(channels) = cores.list()[i].channels() {
                for (direction, channel) in channels.channel() {
                    let packets = *channel.packets_transmitted();
                    if packets != 0 {
                        match direction {
                            Directions::North => {
                                add_to_ret(i, Directions::North);

                                let _ = ManycoreSystem::update_neighbour(
                                    connections.get_mut(&i).ok_or(ConnectionUpdateError)?,
                                    packets as u8,
                                    &Neighbours::top_mut,
                                )?;
                            }
                            Directions::East => {
                                add_to_ret(i, Directions::East);

                                let _ = ManycoreSystem::update_neighbour(
                                    connections.get_mut(&i).ok_or(ConnectionUpdateError)?,
                                    packets as u8,
                                    &Neighbours::right_mut,
                                )?;
                            }
                            Directions::South => {
                                add_to_ret(i, Directions::South);

                                let _ = ManycoreSystem::update_neighbour(
                                    connections.get_mut(&i).ok_or(ConnectionUpdateError)?,
                                    packets as u8,
                                    &Neighbours::bottom_mut,
                                )?;
                            }
                            Directions::West => {
                                add_to_ret(i, Directions::West);

                                let _ = ManycoreSystem::update_neighbour(
                                    connections.get_mut(&i).ok_or(ConnectionUpdateError)?,
                                    packets as u8,
                                    &Neighbours::left_mut,
                                )?;
                            }
                        }
                    }
                }
            }
        }

        Ok(ret)
    }

    /// Clears all links loads.
    fn clear_links(&mut self) {
        // Zero out all links costs
        (&mut self.connections)
            .iter_mut()
            .for_each(|(_, neighbours)| neighbours.clear_link_costs());
    }

    /// Performs routing according to the requested algorithm.
    pub fn route(
        &mut self,
        algorithm: &RoutingAlgorithms,
    ) -> Result<HashMap<usize, Vec<Directions>>, ConnectionUpdateError> {
        self.clear_links();

        match algorithm {
            RoutingAlgorithms::ColumnFirst => self.column_first(),
            RoutingAlgorithms::RowFirst => self.row_first(),
            RoutingAlgorithms::Observed => self.observed_route(),
        }
    }
}