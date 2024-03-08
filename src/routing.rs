use std::{collections::HashMap, error::Error, fmt::Display};

use getset::{Getters, MutGetters, Setters};
use serde::{Deserialize, Serialize};

use crate::{Cores, Edge, FIFODirection, ManycoreSystem, WithXMLAttributes};

#[derive(Default, Debug, PartialEq, Clone, Getters)]
#[getset(get = "pub")]
pub struct Neighbour {
    id: usize,
    link_cost: u8,
}

impl Neighbour {
    fn add_to_cost(&mut self, cost: u8) {
        self.link_cost += cost;
    }
}

impl Neighbour {
    pub fn new(id: Option<usize>) -> Option<Neighbour> {
        if let Some(id) = id {
            return Some(Neighbour { id, link_cost: 0 });
        }

        None
    }
}

#[derive(Default, Debug, PartialEq, Getters, MutGetters, Setters, Clone)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Neighbours {
    top: Option<Neighbour>,
    right: Option<Neighbour>,
    bottom: Option<Neighbour>,
    left: Option<Neighbour>,
}

impl Neighbours {
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

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum RoutingAlgorithms {
    Observed,
    RowFirst,
    ColumnFirst,
}

pub static SUPPORTED_ALGORITHMS: [RoutingAlgorithms; 3] = [
    RoutingAlgorithms::Observed,
    RoutingAlgorithms::RowFirst,
    RoutingAlgorithms::ColumnFirst,
];

struct EdgeRoutingInformation {
    start_id: u8,
    start_column: u8,
    destination_id: u8,
    current_column: u8,
    current_row: u8,
    destination_column: u8,
    destination_row: u8,
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
    fn task_id_to_core_id<'a>(
        task_core_map: &'a HashMap<u16, usize>,
        task_id: &'a u16,
    ) -> Result<&'a usize, ConnectionUpdateError> {
        task_core_map.get(task_id).ok_or(ConnectionUpdateError)
    }

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

    fn update_connection(
        neighbours: &mut Neighbours,
        cost: u8,
        delta_target: &mut u8,
        positive_delta: bool,
        neighbour_selector: &impl Fn(&mut Neighbours) -> &mut Option<Neighbour>,
    ) -> Result<usize, ConnectionUpdateError> {
        let neighbour = ManycoreSystem::update_neighbour(neighbours, cost, neighbour_selector)?;

        if positive_delta {
            *delta_target += 1;
        } else {
            *delta_target -= 1;
        }

        Ok(neighbour.id)
    }

    fn row_first(&mut self) -> Result<HashMap<usize, Vec<FIFODirection>>, ConnectionUpdateError> {
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
        let mut ret: HashMap<usize, Vec<FIFODirection>> = HashMap::new();

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
                        ret.entry(current_idx)
                            .or_insert(Vec::new())
                            .push(FIFODirection::NorthOutput);

                        current_idx = ManycoreSystem::update_connection(
                            neighbours,
                            eri.communication_cost,
                            &mut eri.current_row,
                            false,
                            &Neighbours::top_mut,
                        )?;
                    } else {
                        // Going down
                        ret.entry(current_idx)
                            .or_insert(Vec::new())
                            .push(FIFODirection::SouthOutput);

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
                        ret.entry(current_idx)
                            .or_insert(Vec::new())
                            .push(FIFODirection::WestOutput);

                        current_idx = ManycoreSystem::update_connection(
                            neighbours,
                            eri.communication_cost,
                            &mut eri.current_column,
                            false,
                            &Neighbours::left_mut,
                        )?;
                    } else {
                        // Going right
                        ret.entry(current_idx)
                            .or_insert(Vec::new())
                            .push(FIFODirection::EastOutput);

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

    fn column_first(
        &mut self,
    ) -> Result<HashMap<usize, Vec<FIFODirection>>, ConnectionUpdateError> {
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
        let mut ret: HashMap<usize, Vec<FIFODirection>> = HashMap::new();

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
                        ret.entry(current_idx)
                            .or_insert(Vec::new())
                            .push(FIFODirection::WestOutput);

                        current_idx = ManycoreSystem::update_connection(
                            neighbours,
                            eri.communication_cost,
                            &mut eri.current_column,
                            false,
                            &Neighbours::left_mut,
                        )?;
                    } else {
                        // Going right
                        ret.entry(current_idx)
                            .or_insert(Vec::new())
                            .push(FIFODirection::EastOutput);

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
                        ret.entry(current_idx)
                            .or_insert(Vec::new())
                            .push(FIFODirection::NorthOutput);

                        current_idx = ManycoreSystem::update_connection(
                            neighbours,
                            eri.communication_cost,
                            &mut eri.current_row,
                            false,
                            &Neighbours::top_mut,
                        )?;
                    } else {
                        // Going down
                        ret.entry(current_idx)
                            .or_insert(Vec::new())
                            .push(FIFODirection::SouthOutput);

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

    fn clear_links(&mut self) {
        // Zero out all links costs
        (&mut self.connections)
            .iter_mut()
            .for_each(|(_, neighbours)| neighbours.clear_link_costs());
    }

    pub fn task_graph_route(
        &mut self,
        algorithm: &RoutingAlgorithms,
    ) -> Result<HashMap<usize, Vec<FIFODirection>>, ConnectionUpdateError> {
        self.clear_links();

        match algorithm {
            RoutingAlgorithms::ColumnFirst => self.column_first(),
            RoutingAlgorithms::RowFirst | _ => self.row_first(),
        }
    }

    pub fn observed_route(
        &mut self,
    ) -> Result<HashMap<usize, Vec<FIFODirection>>, ConnectionUpdateError> {
        self.clear_links();
        let ManycoreSystem {
            ref cores,
            ref mut connections,
            ..
        } = *self;

        let mut ret: HashMap<usize, Vec<FIFODirection>> = HashMap::new();

        for i in 0..cores.list().len() {
            if let Some(fifos) = cores.list()[i].fifos() {
                for (direction, fifo) in fifos.fifo() {
                    let packets = *fifo.packets_transmitted();
                    if packets != 0 {
                        match direction {
                            FIFODirection::NorthOutput => {
                                ret.entry(i)
                                    .or_insert(Vec::new())
                                    .push(FIFODirection::NorthOutput);
                                let _ = ManycoreSystem::update_neighbour(
                                    connections.get_mut(&i).ok_or(ConnectionUpdateError)?,
                                    packets as u8,
                                    &Neighbours::top_mut,
                                )?;
                            }
                            FIFODirection::EastOutput => {
                                ret.entry(i)
                                    .or_insert(Vec::new())
                                    .push(FIFODirection::EastOutput);

                                let _ = ManycoreSystem::update_neighbour(
                                    connections.get_mut(&i).ok_or(ConnectionUpdateError)?,
                                    packets as u8,
                                    &Neighbours::right_mut,
                                )?;
                            }
                            FIFODirection::SouthOutput => {
                                ret.entry(i)
                                    .or_insert(Vec::new())
                                    .push(FIFODirection::SouthOutput);

                                let _ = ManycoreSystem::update_neighbour(
                                    connections.get_mut(&i).ok_or(ConnectionUpdateError)?,
                                    packets as u8,
                                    &Neighbours::bottom_mut,
                                )?;
                            }
                            FIFODirection::WestOutput => {
                                ret.entry(i)
                                    .or_insert(Vec::new())
                                    .push(FIFODirection::WestOutput);

                                let _ = ManycoreSystem::update_neighbour(
                                    connections.get_mut(&i).ok_or(ConnectionUpdateError)?,
                                    packets as u8,
                                    &Neighbours::left_mut,
                                )?;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use crate::ManycoreSystem;

    use super::{Neighbour, Neighbours};

    fn get_neighbour_by_id<'a>(
        manycore: &'a ManycoreSystem,
        id: &usize,
        neighbour_selector: &impl Fn(&Neighbours) -> &Option<Neighbour>,
        // Impl instead of dyn because there are only 4 variants of the function
        // so it's okay for the compiler to generate the 4 signatures.
    ) -> &'a Neighbour {
        &neighbour_selector(
            &manycore
                .connections()
                .get(id)
                .expect(&format!("Could not get connections for ID {}", *id)),
        )
        .as_ref()
        .expect(&format!("Could not get wanted neighbour for ID {}", id))
    }

    #[test]
    fn row_first_is_correct() {
        let mut manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
            .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

        manycore
            .task_graph_route(&super::RoutingAlgorithms::RowFirst)
            .unwrap();

        // Do the routing by hand to verify these, no other way really
        assert_eq!(
            3,
            *get_neighbour_by_id(&manycore, &6, &Neighbours::top).link_cost()
        );
        assert_eq!(
            3,
            *get_neighbour_by_id(&manycore, &3, &Neighbours::right).link_cost()
        );
        assert_eq!(
            2,
            *get_neighbour_by_id(&manycore, &6, &Neighbours::right).link_cost()
        );
        assert_eq!(
            4,
            *get_neighbour_by_id(&manycore, &4, &Neighbours::top).link_cost()
        );
        assert_eq!(
            1,
            *get_neighbour_by_id(&manycore, &7, &Neighbours::top).link_cost()
        );
    }

    #[test]
    fn column_first_is_correct() {
        let mut manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
            .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

        manycore
            .task_graph_route(&super::RoutingAlgorithms::ColumnFirst)
            .unwrap();

        // Do the routing by hand to verify these, no other way really
        assert_eq!(
            5,
            *get_neighbour_by_id(&manycore, &6, &Neighbours::right).link_cost()
        );
        assert_eq!(
            4,
            *get_neighbour_by_id(&manycore, &7, &Neighbours::top).link_cost()
        );
        assert_eq!(
            4,
            *get_neighbour_by_id(&manycore, &4, &Neighbours::top).link_cost()
        );
    }
}
