// #[cfg(test)]
// use crate::{ManycoreSystem, Neighbour, Neighbours, RoutingAlgorithms};

// #[cfg(test)]
// fn get_neighbour_by_id<'a>(
//     manycore: &'a ManycoreSystem,
//     id: &usize,
//     neighbour_selector: &impl Fn(&Neighbours) -> &Option<Neighbour>,
//     // Impl instead of dyn because there are only 4 variants of the function
//     // so it's okay for the compiler to generate the 4 signatures.
// ) -> &'a Neighbour {
//     &neighbour_selector(
//         &manycore
//             .connections()
//             .get(id)
//             .expect(&format!("Could not get connections for ID {}", *id)),
//     )
//     .as_ref()
//     .expect(&format!("Could not get wanted neighbour for ID {}", id))
// }

// #[test]
// fn row_first_is_correct() {
//     let mut manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
//         .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

//     manycore.route(&RoutingAlgorithms::RowFirst).unwrap();

//     // Do the routing by hand to verify these, no other way really
//     assert_eq!(
//         3,
//         *get_neighbour_by_id(&manycore, &6, &Neighbours::top).link_cost()
//     );
//     assert_eq!(
//         3,
//         *get_neighbour_by_id(&manycore, &3, &Neighbours::right).link_cost()
//     );
//     assert_eq!(
//         2,
//         *get_neighbour_by_id(&manycore, &6, &Neighbours::right).link_cost()
//     );
//     assert_eq!(
//         4,
//         *get_neighbour_by_id(&manycore, &4, &Neighbours::top).link_cost()
//     );
//     assert_eq!(
//         1,
//         *get_neighbour_by_id(&manycore, &7, &Neighbours::top).link_cost()
//     );
// }

// #[test]
// fn column_first_is_correct() {
//     let mut manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
//         .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

//     manycore.route(&RoutingAlgorithms::ColumnFirst).unwrap();

//     // Do the routing by hand to verify these, no other way really
//     assert_eq!(
//         5,
//         *get_neighbour_by_id(&manycore, &6, &Neighbours::right).link_cost()
//     );
//     assert_eq!(
//         4,
//         *get_neighbour_by_id(&manycore, &7, &Neighbours::top).link_cost()
//     );
//     assert_eq!(
//         4,
//         *get_neighbour_by_id(&manycore, &4, &Neighbours::top).link_cost()
//     );
// }
