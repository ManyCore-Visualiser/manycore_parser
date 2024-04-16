#[cfg(test)]
use crate::{
    get_core, routing_error, Directions, ManycoreError, ManycoreSystem, RoutingAlgorithms,
};

#[cfg(test)]
fn get_load(
    manycore: &mut ManycoreSystem,
    id: usize,
    direction: Directions,
) -> Result<u16, ManycoreError> {
    Ok(*get_core(manycore.cores_mut(), id)?
        .channels()
        .channel()
        .get(&direction)
        .ok_or(routing_error(format!(
            "Core {id} has no {direction} channel."
        )))?
        .current_load())
}

#[cfg(test)]
fn get_source_load(
    manycore: &mut ManycoreSystem,
    id: usize,
    direction: Directions,
) -> Result<u16, ManycoreError> {
    Ok(*get_core(manycore.cores_mut(), id)?
        .source_loads()
        .as_ref()
        .ok_or(routing_error(format!("Core {id} has no source loads")))?
        .get(&direction)
        .ok_or(routing_error(format!(
            "Core {id} has no {direction} source channel."
        )))?)
}

#[test]
fn row_first_is_correct() {
    let mut manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
        .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

    manycore.route(&RoutingAlgorithms::RowFirst).unwrap();

    // Do the routing by hand to verify these, no other way really
    assert_eq!(20, get_load(&mut manycore, 0, Directions::South).unwrap());
    assert_eq!(20, get_load(&mut manycore, 3, Directions::South).unwrap());
    assert_eq!(180, get_load(&mut manycore, 1, Directions::South).unwrap());
    assert_eq!(80, get_load(&mut manycore, 4, Directions::South).unwrap());
    assert_eq!(50, get_load(&mut manycore, 4, Directions::North).unwrap());
    assert_eq!(50, get_load(&mut manycore, 7, Directions::North).unwrap());
    assert_eq!(100, get_load(&mut manycore, 4, Directions::East).unwrap());
    assert_eq!(30, get_load(&mut manycore, 8, Directions::West).unwrap());
    assert_eq!(80, get_load(&mut manycore, 7, Directions::West).unwrap());
    assert_eq!(80, get_load(&mut manycore, 6, Directions::West).unwrap());
    assert_eq!(20, get_load(&mut manycore, 6, Directions::East).unwrap());
    assert_eq!(
        20,
        get_source_load(&mut manycore, 0, Directions::West).unwrap()
    );
    assert_eq!(
        30,
        get_source_load(&mut manycore, 1, Directions::North).unwrap()
    );
}

#[test]
fn column_first_is_correct() {
    let mut manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
        .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

    manycore.route(&RoutingAlgorithms::ColumnFirst).unwrap();

    // Do the routing by hand to verify these, no other way really
    assert_eq!(20, get_load(&mut manycore, 0, Directions::East).unwrap());
    assert_eq!(50, get_load(&mut manycore, 0, Directions::South).unwrap());
    assert_eq!(100, get_load(&mut manycore, 1, Directions::East).unwrap());
    assert_eq!(50, get_load(&mut manycore, 1, Directions::South).unwrap());
    assert_eq!(80, get_load(&mut manycore, 3, Directions::South).unwrap());
    assert_eq!(50, get_load(&mut manycore, 4, Directions::South).unwrap());
    assert_eq!(50, get_load(&mut manycore, 1, Directions::West).unwrap());
    assert_eq!(30, get_load(&mut manycore, 4, Directions::West).unwrap());
    assert_eq!(30, get_load(&mut manycore, 5, Directions::West).unwrap());
    assert_eq!(80, get_load(&mut manycore, 6, Directions::West).unwrap());
    assert_eq!(50, get_load(&mut manycore, 7, Directions::North).unwrap());
    assert_eq!(50, get_load(&mut manycore, 4, Directions::North).unwrap());
    assert_eq!(
        20,
        get_source_load(&mut manycore, 0, Directions::West).unwrap()
    );
    assert_eq!(
        30,
        get_source_load(&mut manycore, 1, Directions::North).unwrap()
    );
}
