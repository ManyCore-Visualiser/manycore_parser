#[cfg(test)]
use std::{
    collections::{BTreeMap, HashMap},
    fs::read_to_string,
};

#[cfg(test)]
use crate::{
    sink::Sink, source::Source, AttributeType, Borders, Channel, ChannelStatus, Channels,
    ConfigurableAttributes, Core, Cores, Directions, Edge, ManycoreSystem, Router,
    SinkSourceDirection, Task, TaskGraph, WithXMLAttributes, SUPPORTED_ALGORITHMS,
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

    let expected_channels = Channels::new(BTreeMap::from([
        (
            Directions::North,
            Channel::new(Directions::North, 30, 4, ChannelStatus::Normal, 400),
        ),
        (
            Directions::South,
            Channel::new(Directions::South, 30, 4, ChannelStatus::Normal, 400),
        ),
        (
            Directions::East,
            Channel::new(Directions::East, 30, 4, ChannelStatus::Normal, 400),
        ),
        (
            Directions::West,
            Channel::new(Directions::West, 30, 0, ChannelStatus::Normal, 400),
        ),
    ]));

    let mut expected_router = Router::new(
        0,
        Some(BTreeMap::from([
            ("@age".to_string(), "30".to_string()),
            ("@temperature".to_string(), "30".to_string()),
            ("@status".to_string(), "Normal".to_string()),
        ])),
    );

    let expected_cores = vec![
        Core::new(
            0,
            expected_router.clone(),
            None,
            expected_channels.clone(),
            Some(BTreeMap::from([
                ("@age".to_string(), "238".to_string()),
                ("@temperature".to_string(), "45".to_string()),
                ("@status".to_string(), "High".to_string()),
                ("@actualFrequency".to_string(), "Low".to_string()),
            ])),
        ),
        Core::new(
            1,
            expected_router.clone_increment(),
            Some(3),
            expected_channels.clone(),
            Some(BTreeMap::from([
                ("@age".to_string(), "394".to_string()),
                ("@temperature".to_string(), "30".to_string()),
                ("@status".to_string(), "High".to_string()),
                ("@actualFrequency".to_string(), "High".to_string()),
            ])),
        ),
        Core::new(
            2,
            expected_router.clone_increment(),
            None,
            expected_channels.clone(),
            Some(BTreeMap::from([
                ("@age".to_string(), "157".to_string()),
                ("@temperature".to_string(), "30".to_string()),
                ("@status".to_string(), "High".to_string()),
                ("@actualFrequency".to_string(), "Low".to_string()),
            ])),
        ),
        Core::new(
            3,
            expected_router.clone_increment(),
            None,
            expected_channels.clone(),
            Some(BTreeMap::from([
                ("@age".to_string(), "225".to_string()),
                ("@temperature".to_string(), "30".to_string()),
                ("@status".to_string(), "High".to_string()),
                ("@actualFrequency".to_string(), "Low".to_string()),
            ])),
        ),
        Core::new(
            4,
            expected_router.clone_increment(),
            None,
            expected_channels.clone(),
            Some(BTreeMap::from([
                ("@age".to_string(), "478".to_string()),
                ("@temperature".to_string(), "30".to_string()),
                ("@status".to_string(), "High".to_string()),
                ("@actualFrequency".to_string(), "High".to_string()),
            ])),
        ),
        Core::new(
            5,
            expected_router.clone_increment(),
            Some(4),
            expected_channels.clone(),
            Some(BTreeMap::from([
                ("@age".to_string(), "105".to_string()),
                ("@temperature".to_string(), "30".to_string()),
                ("@status".to_string(), "High".to_string()),
                ("@actualFrequency".to_string(), "Low".to_string()),
            ])),
        ),
        Core::new(
            6,
            expected_router.clone_increment(),
            None,
            expected_channels.clone(),
            Some(BTreeMap::from([
                ("@age".to_string(), "18".to_string()),
                ("@temperature".to_string(), "30".to_string()),
                ("@status".to_string(), "High".to_string()),
                ("@actualFrequency".to_string(), "High".to_string()),
            ])),
        ),
        Core::new(
            7,
            expected_router.clone_increment(),
            Some(2),
            expected_channels.clone(),
            Some(BTreeMap::from([
                ("@age".to_string(), "15".to_string()),
                ("@temperature".to_string(), "30".to_string()),
                ("@status".to_string(), "High".to_string()),
                ("@actualFrequency".to_string(), "Mid".to_string()),
            ])),
        ),
        Core::new(
            8,
            expected_router.clone_increment(),
            None,
            expected_channels.clone(),
            Some(BTreeMap::from([
                ("@age".to_string(), "10".to_string()),
                ("@temperature".to_string(), "30".to_string()),
                ("@status".to_string(), "High".to_string()),
                ("@actualFrequency".to_string(), "Low".to_string()),
            ])),
        ),
    ];

    let expected_configurable_attributes = ConfigurableAttributes {
        core: BTreeMap::from([
            ("@id".to_string(), AttributeType::Text),
            ("@coordinates".to_string(), AttributeType::Text),
            ("@age".to_string(), AttributeType::Number),
            ("@temperature".to_string(), AttributeType::Number),
            ("@status".to_string(), AttributeType::Text),
            ("@actualFrequency".to_string(), AttributeType::Text),
        ]),
        router: BTreeMap::from([
            ("@age".to_string(), AttributeType::Number),
            ("@temperature".to_string(), AttributeType::Number),
            ("@status".to_string(), AttributeType::Text),
        ]),
        algorithms: Vec::from(&SUPPORTED_ALGORITHMS),
        observed_algorithm: Some(String::from("RowFirst")),
        sinks_sources: true,
    };

    let expected_task_core_map = HashMap::from([(3u16, 1usize), (2u16, 7usize), (4u16, 5usize)]);

    let expected_sinks = BTreeMap::from([(5, Sink::new(6, SinkSourceDirection::West, 5))]);
    let expected_sources = BTreeMap::from([
        (0, Source::new(1, SinkSourceDirection::North, 0)),
        (1, Source::new(0, SinkSourceDirection::West, 1)),
    ]);
    let expected_core_source_map = HashMap::from([
        (1, HashMap::from([(SinkSourceDirection::North, vec![0])])),
        (0, HashMap::from([(SinkSourceDirection::West, vec![1])])),
    ]);

    let expected_manycore = ManycoreSystem {
        xmlns: String::from(
            "https://www.york.ac.uk/physics-engineering-technology/ManycoreSystems",
        ),
        xmlns_si: String::from("http://www.w3.org/2001/XMLSchema-instance"),
        xsi_schema_location: String::from("https://www.york.ac.uk/physics-engineering-technology/ManycoreSystems https://gist.githubusercontent.com/joe2k01/718e437790047ca14447af3b8309ef76/raw/057205e76461d12f33c7a54b27d5b2c99d57d9a8/manycore_schema.xsd"),
        columns: 3,
        rows: 3,
        routing_algo: Some(String::from("RowFirst")),
        borders: Borders::new(expected_sinks, expected_sources, expected_core_source_map),
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

    let res = quick_xml::se::to_string(&manycore).expect("Could not serialize ManyCore");

    let expected = read_to_string("tests/serialized.xml")
        .expect("Could not read input test file \"tests/serialized.xml\"");

    assert_eq!(res, expected)
}
