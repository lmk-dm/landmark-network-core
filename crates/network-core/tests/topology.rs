use network_core::{
    Edge, EdgeKind, FluidService, NetworkError, NetworkModel, Node, NodeKind,
};

fn mini_gathering() -> NetworkModel {
    let mut m = NetworkModel::new("permian-demo").with_as_of("2026-11-26");
    m.add_node(Node::new("w1", NodeKind::Well).with_fluid(FluidService::Multiphase))
        .unwrap();
    m.add_node(Node::new("j1", NodeKind::Junction)).unwrap();
    m.add_node(
        Node::new("bat1", NodeKind::Facility)
            .with_fluid(FluidService::Oil)
            .with_name("North Battery"),
    )
    .unwrap();
    m.add_node(Node::new("sales", NodeKind::Sink).with_fluid(FluidService::Oil))
        .unwrap();

    m.add_edge(Edge::new(
        "l1",
        EdgeKind::Pipe,
        "w1",
        "j1",
        FluidService::Multiphase,
    ))
    .unwrap();
    m.add_edge(Edge::new(
        "l2",
        EdgeKind::Pipe,
        "j1",
        "bat1",
        FluidService::Multiphase,
    ))
    .unwrap();
    m.add_edge(Edge::new(
        "oil-out",
        EdgeKind::Pipe,
        "bat1",
        "sales",
        FluidService::Oil,
    ))
    .unwrap();
    m
}

#[test]
fn valid_gathering_passes() {
    let m = mini_gathering();
    assert!(m.validate().is_ok());
    assert_eq!(m.nodes.len(), 4);
    assert_eq!(m.edges.len(), 3);
}

#[test]
fn empty_network_refused() {
    let m = NetworkModel::new("empty");
    assert!(matches!(m.validate(), Err(NetworkError::EmptyNetwork)));
}

#[test]
fn dangling_edge_refused() {
    let mut m = NetworkModel::new("broken");
    m.add_node(Node::new("w1", NodeKind::Well)).unwrap();
    m.add_edge(Edge::new(
        "l1",
        EdgeKind::Pipe,
        "w1",
        "missing",
        FluidService::Oil,
    ))
    .unwrap();
    match m.validate() {
        Err(NetworkError::MissingNode { edge, node }) => {
            assert_eq!(edge, "l1");
            assert_eq!(node, "missing");
        }
        other => panic!("expected MissingNode, got {other:?}"),
    }
}

#[test]
fn duplicate_node_refused_on_insert() {
    let mut m = NetworkModel::new("dup");
    m.add_node(Node::new("w1", NodeKind::Well)).unwrap();
    let err = m.add_node(Node::new("w1", NodeKind::Facility)).unwrap_err();
    assert!(matches!(err, NetworkError::DuplicateNode(_)));
}

#[test]
fn self_loop_refused_on_insert() {
    let mut m = NetworkModel::new("loop");
    m.add_node(Node::new("j1", NodeKind::Junction)).unwrap();
    let err = m
        .add_edge(Edge::new(
            "bad",
            EdgeKind::Valve,
            "j1",
            "j1",
            FluidService::Gas,
        ))
        .unwrap_err();
    assert!(matches!(err, NetworkError::SelfLoop(_)));
}

#[test]
fn topology_hash_ignores_display_name() {
    let a = mini_gathering();
    let mut b = mini_gathering();
    if let Some(n) = b.nodes.iter_mut().find(|n| n.id.as_str() == "bat1") {
        n.name = Some("Renamed Battery".into());
    }
    assert_eq!(a.topology_sha256(), b.topology_sha256());
}

#[test]
fn topology_hash_changes_when_edge_added() {
    let a = mini_gathering();
    let mut b = mini_gathering();
    b.add_node(Node::new("w2", NodeKind::Well)).unwrap();
    b.add_edge(Edge::new(
        "l3",
        EdgeKind::Pipe,
        "w2",
        "j1",
        FluidService::Multiphase,
    ))
    .unwrap();
    assert_ne!(a.topology_sha256(), b.topology_sha256());
}

#[test]
fn serde_round_trip() {
    let m = mini_gathering();
    let json = serde_json::to_string_pretty(&m).unwrap();
    let back: NetworkModel = serde_json::from_str(&json).unwrap();
    assert_eq!(m, back);
    assert_eq!(m.topology_sha256(), back.topology_sha256());
}
