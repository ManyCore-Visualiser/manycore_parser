#[cfg(test)]
use std::{
    collections::{BTreeMap, HashMap},
    fs::read_to_string,
};

#[cfg(test)]
use crate::{
    AttributeType, AttributesMap, BorderEntry, Borders, Channel, Channels, ConfigurableAttributes,
    Core, Cores, Directions, Edge, ManycoreSystem, ProcessedAttribute, Router, Sink,
    SinkSourceDirection, Source, Task, TaskGraph, WithID, BORDER_ROUTERS_KEY, COORDINATES_KEY,
    ID_KEY, ROUTING_KEY, SUPPORTED_ALGORITHMS, TASK_COST_KEY,
};

#[cfg(test)]
impl Router {
    fn clone_increment(&mut self) -> Self {
        self.set_id(self.id() + 1);

        self.clone()
    }
}

#[test]
fn can_parse() {
    let age_string = "@age".to_string();
    let temperature_string = "@temperature".to_string();
    let status_string = "@status".to_string();
    let acc_freq_string = "@actualFrequency".to_string();

    let expected_tasks = vec![Task::new(2, 40), Task::new(3, 80), Task::new(4, 60)];

    let expected_edges = vec![
        Edge::new(0, 2, 30),
        Edge::new(1, 2, 20),
        Edge::new(2, 3, 50),
        Edge::new(3, 4, 100),
        Edge::new(3, 5, 50),
        Edge::new(4, 5, 30),
    ];

    let expected_graph = TaskGraph::new(expected_tasks, expected_edges);
    let expected_channel_attributes = BTreeMap::from([
        (age_string.clone(), "30".into()),
        (status_string.clone(), "Normal".into()),
    ]);

    let expected_channels = Channels::new(BTreeMap::from([
        (
            Directions::North,
            Channel::new(
                Directions::North,
                4,
                400,
                Some(expected_channel_attributes.clone()),
            ),
        ),
        (
            Directions::South,
            Channel::new(
                Directions::South,
                4,
                400,
                Some(expected_channel_attributes.clone()),
            ),
        ),
        (
            Directions::East,
            Channel::new(
                Directions::East,
                4,
                400,
                Some(expected_channel_attributes.clone()),
            ),
        ),
        (
            Directions::West,
            Channel::new(
                Directions::West,
                0,
                400,
                Some(expected_channel_attributes.clone()),
            ),
        ),
    ]));

    let mut expected_router = Router::new(
        0,
        Some(BTreeMap::from([
            (age_string.clone(), "30".to_string()),
            (temperature_string.clone(), "30".to_string()),
            (status_string.clone(), "Normal".to_string()),
        ])),
    );

    let expected_columns = 3;
    let expected_rows = 3;

    let expected_cores = vec![
        Core::new(
            0,
            expected_columns,
            expected_rows,
            expected_router.clone(),
            None,
            expected_channels.clone(),
            Some(BTreeMap::from([
                (age_string.clone(), "238".to_string()),
                (temperature_string.clone(), "45".to_string()),
                (status_string.clone(), "High".to_string()),
                (acc_freq_string.clone(), "Low".to_string()),
            ])),
        ),
        Core::new(
            1,
            expected_columns,
            expected_rows,
            expected_router.clone_increment(),
            Some(3),
            expected_channels.clone(),
            Some(BTreeMap::from([
                (age_string.clone(), "394".to_string()),
                (temperature_string.clone(), "30".to_string()),
                (status_string.clone(), "High".to_string()),
                (acc_freq_string.clone(), "High".to_string()),
            ])),
        ),
        Core::new(
            2,
            expected_columns,
            expected_rows,
            expected_router.clone_increment(),
            None,
            expected_channels.clone(),
            Some(BTreeMap::from([
                (age_string.clone(), "157".to_string()),
                (temperature_string.clone(), "30".to_string()),
                (status_string.clone(), "High".to_string()),
                (acc_freq_string.clone(), "Low".to_string()),
            ])),
        ),
        Core::new(
            3,
            expected_columns,
            expected_rows,
            expected_router.clone_increment(),
            None,
            expected_channels.clone(),
            Some(BTreeMap::from([
                (age_string.clone(), "225".to_string()),
                (temperature_string.clone(), "30".to_string()),
                (status_string.clone(), "High".to_string()),
                (acc_freq_string.clone(), "Low".to_string()),
            ])),
        ),
        Core::new(
            4,
            expected_columns,
            expected_rows,
            expected_router.clone_increment(),
            None,
            expected_channels.clone(),
            Some(BTreeMap::from([
                (age_string.clone(), "478".to_string()),
                (temperature_string.clone(), "30".to_string()),
                (status_string.clone(), "High".to_string()),
                (acc_freq_string.clone(), "High".to_string()),
            ])),
        ),
        Core::new(
            5,
            expected_columns,
            expected_rows,
            expected_router.clone_increment(),
            Some(4),
            expected_channels.clone(),
            Some(BTreeMap::from([
                (age_string.clone(), "105".to_string()),
                (temperature_string.clone(), "30".to_string()),
                (status_string.clone(), "High".to_string()),
                (acc_freq_string.clone(), "Low".to_string()),
            ])),
        ),
        Core::new(
            6,
            expected_columns,
            expected_rows,
            expected_router.clone_increment(),
            None,
            expected_channels.clone(),
            Some(BTreeMap::from([
                (age_string.clone(), "18".to_string()),
                (temperature_string.clone(), "30".to_string()),
                (status_string.clone(), "High".to_string()),
                (acc_freq_string.clone(), "High".to_string()),
            ])),
        ),
        Core::new(
            7,
            expected_columns,
            expected_rows,
            expected_router.clone_increment(),
            Some(2),
            expected_channels.clone(),
            Some(BTreeMap::from([
                (age_string.clone(), "15".to_string()),
                (temperature_string.clone(), "30".to_string()),
                (status_string.clone(), "High".to_string()),
                (acc_freq_string.clone(), "Mid".to_string()),
            ])),
        ),
        Core::new(
            8,
            expected_columns,
            expected_rows,
            expected_router.clone_increment(),
            None,
            expected_channels.clone(),
            Some(BTreeMap::from([
                (age_string.clone(), "10".to_string()),
                (temperature_string.clone(), "30".to_string()),
                (status_string.clone(), "High".to_string()),
                (acc_freq_string.clone(), "Low".to_string()),
            ])),
        ),
    ];

    let mut expected_core_conf_attrs = BTreeMap::from([
        (
            age_string.clone(),
            ProcessedAttribute::new(&age_string, AttributeType::Number),
        ),
        (
            temperature_string.clone(),
            ProcessedAttribute::new(&temperature_string, AttributeType::Number),
        ),
        (
            status_string.clone(),
            ProcessedAttribute::new(&status_string, AttributeType::Text),
        ),
        (
            acc_freq_string.clone(),
            ProcessedAttribute::new(&acc_freq_string, AttributeType::Text),
        ),
    ]);
    expected_core_conf_attrs.insert_manual(ID_KEY, AttributeType::Text);
    expected_core_conf_attrs.insert_manual(COORDINATES_KEY, AttributeType::Coordinates);
    expected_core_conf_attrs.insert_manual(TASK_COST_KEY, AttributeType::Boolean);

    let expected_router_conf_attrs = BTreeMap::from([
        (
            age_string.clone(),
            ProcessedAttribute::new(&age_string, AttributeType::Number),
        ),
        (
            temperature_string.clone(),
            ProcessedAttribute::new(&temperature_string, AttributeType::Number),
        ),
        (
            status_string.clone(),
            ProcessedAttribute::new(&status_string, AttributeType::Text),
        ),
    ]);

    let mut expected_channel_conf_attrs = BTreeMap::from([
        (
            age_string.clone(),
            ProcessedAttribute::new(&age_string, AttributeType::Number),
        ),
        (
            status_string.clone(),
            ProcessedAttribute::new(&status_string, AttributeType::Text),
        ),
    ]);
    expected_channel_conf_attrs.insert_manual(ROUTING_KEY, AttributeType::Routing);
    expected_channel_conf_attrs.insert_manual(BORDER_ROUTERS_KEY, AttributeType::Boolean);

    let expected_configurable_attributes = ConfigurableAttributes::new(
        expected_core_conf_attrs,
        expected_router_conf_attrs,
        Some(String::from("RowFirst")),
        Vec::from(&SUPPORTED_ALGORITHMS),
        expected_channel_conf_attrs,
    );

    let expected_task_core_map = HashMap::from([(3u16, 1usize), (2u16, 7usize), (4u16, 5usize)]);

    let expected_sinks = BTreeMap::from([(5, Sink::new(6, SinkSourceDirection::West, 5))]);
    let expected_sources = BTreeMap::from([
        (0, Source::new(1, SinkSourceDirection::North, 0, Some(10))),
        (1, Source::new(0, SinkSourceDirection::West, 1, None)),
    ]);
    let expected_core_border_map = HashMap::from([
        (
            6,
            HashMap::from([(SinkSourceDirection::West, BorderEntry::Sink(5))]),
        ),
        (
            1,
            HashMap::from([(SinkSourceDirection::North, BorderEntry::Source(0))]),
        ),
        (
            0,
            HashMap::from([(SinkSourceDirection::West, BorderEntry::Source(1))]),
        ),
    ]);

    let expected_manycore = ManycoreSystem {
        xmlns: String::from(
            "https://www.york.ac.uk/physics-engineering-technology/ManycoreSystems",
        ),
        xmlns_si: String::from("http://www.w3.org/2001/XMLSchema-instance"),
        xsi_schema_location: String::from("https://www.york.ac.uk/physics-engineering-technology/ManycoreSystems https://gist.githubusercontent.com/joe2k01/718e437790047ca14447af3b8309ef76/raw/3e0d9d40ecead18fe3967b831160edd3463908d1/manycore_schema.xsd"),
        columns: expected_columns,
        rows: expected_rows,
        routing_algo: Some(String::from("RowFirst")),
        borders: Some(Borders::new(expected_sinks, expected_sources, expected_core_border_map)),
        cores: Cores::new(expected_cores),
        task_graph: expected_graph,
        task_core_map: expected_task_core_map,
        configurable_attributes: expected_configurable_attributes
    };

    let manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
        .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

    assert_eq!(manycore, expected_manycore)
}

#[test]
fn can_serialize() {
    let manycore = ManycoreSystem::parse_file("tests/VisualiserOutput1.xml")
        .expect("Could not read input test file \"tests/VisualiserOutput1.xml\"");

    let res = String::try_from(&manycore).expect("Could not serialize ManyCore");

    let expected = read_to_string("tests/serialized.xml")
        .expect("Could not read input test file \"tests/serialized.xml\"");

    assert_eq!(res, expected)
    // println!("{res}")
}

#[test]
fn can_validate() {
    assert!(ManycoreSystem::parse_file("tests/Validation0.xml").is_err());
    assert!(ManycoreSystem::parse_file("tests/Validation1.xml").is_err())
}
