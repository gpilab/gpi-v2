use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{nodes::status::NodeError, StableMap};

type WireDataContainer<T> = Arc<RwLock<T>>;

pub trait GraphNode<NodeData, PortType, WireData>
where
    PortType: Clone,
{
    fn inputs(&self) -> StableMap<String, PortType>;
    fn outputs(&self) -> StableMap<String, PortType>;
    fn compute(
        self,
        inputs: StableMap<String, WireDataContainer<WireData>>,
    ) -> Result<(StableMap<String, WireData>, NodeData), NodeError>;
}

type PortName = String;

type NodeIndex = u32;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum IO {
    In,
    Out,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct PortRef {
    pub node: u32,
    pub name: PortName,
    pub io: IO,
}

type Edge = (PortRef, PortRef);

#[derive(Serialize, Deserialize)]
pub struct Graph<NodeData, PortType, WireData>
where
    NodeData: GraphNode<NodeData, PortType, WireData>,
    PortType: Clone,
    WireData: std::fmt::Debug,
{
    nodes: crate::StableMap<NodeIndex, NodeData>,
    edges: Vec<Edge>,
    #[serde(skip, default = "default_wire_data")]
    wire_data: HashMap<(NodeIndex, PortName), WireDataContainer<WireData>>,
    next_id: NodeIndex,
    #[serde(skip)]
    phantom: std::marker::PhantomData<PortType>,
}

impl<NodeData: Clone, PortType: Clone, WireData> Clone for Graph<NodeData, PortType, WireData>
where
    NodeData: GraphNode<NodeData, PortType, WireData>,
    PortType: Clone,
    WireData: std::fmt::Debug,
{
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
            wire_data: Default::default(),
            next_id: self.next_id,
            phantom: self.phantom,
        }
    }
}

fn default_wire_data<K, V>() -> HashMap<K, V> {
    HashMap::new()
}
impl<NodeData, PortType, WireData> Graph<NodeData, PortType, WireData>
where
    NodeData: GraphNode<NodeData, PortType, WireData> + Clone,
    PortType: Clone,
    WireData: std::fmt::Debug,
{
    pub fn new() -> Self {
        Self {
            nodes: StableMap::new(),
            edges: vec![],
            wire_data: HashMap::new(),
            next_id: 0,
            phantom: std::marker::PhantomData,
        }
    }

    /// Add a new node to the graph, returns the node's index
    pub fn node(&mut self, node: NodeData) -> NodeIndex {
        let id = self.next_id;
        self.nodes.insert(id, node);
        self.next_id += 1;
        id
    }

    /// Remove a node and all edges associated with it
    pub fn delete_node(&mut self, id: NodeIndex) {
        self.nodes.remove(&id);
        self.edges
            .retain(|(from, to)| from.node != id && to.node != id)
    }
    ///Get the node value at a given index
    ///panics if index is not valid!
    ///Use the index returned from `add_node` to ensure it exists
    pub fn get_node(&self, nx: NodeIndex) -> &NodeData {
        &self.nodes[&nx]
    }

    ///Get a mutable reference  to a node value at a given index
    ///panics if index is not valid
    ///Use the index returned from `add_node` to ensure it exists
    pub fn get_mut_node(&mut self, nx: NodeIndex) -> &mut NodeData {
        self.nodes.get_mut(&nx).unwrap()
    }
    pub fn get_output_data(
        &self,
        nx: NodeIndex,
    ) -> StableMap<String, Option<&WireDataContainer<WireData>>> {
        self.get_node(nx)
            .outputs()
            .clone()
            .into_keys()
            .map(|port_name| {
                (
                    port_name.clone(),
                    self.wire_data.get(&(nx, port_name.clone())),
                )
            })
            .collect()
    }
    pub fn get_input_data(&self, nx: &NodeIndex) -> StableMap<String, WireDataContainer<WireData>> {
        self.get_node(*nx)
            .inputs()
            .keys()
            .filter_map(|port_name| {
                self.get_parent(nx, port_name.clone()).map(|out_port| {
                    self.wire_data
                        .get(&(out_port.node, out_port.name))
                        .map(|data| (port_name.clone(), data.clone()))
                })
            })
            .collect::<Option<StableMap<_, _>>>()
            .unwrap_or([].into())
    }
    pub fn get_input_data_mapped(
        &self,
        nx: &NodeIndex,
    ) -> (
        StableMap<String, String>,
        StableMap<String, WireDataContainer<WireData>>,
    ) {
        let inputs = self.get_node(*nx).inputs();

        let data_with_duplicates = inputs
            .keys()
            .filter_map(|port_name| {
                self.get_parent(nx, port_name.clone()).map(|out_port| {
                    self.wire_data
                        .get(&(out_port.node, out_port.name))
                        .map(|data| (port_name.clone(), data))
                })
            })
            .collect::<Option<StableMap<_, _>>>()
            .unwrap_or([].into());

        let port_matches: StableMap<String, String> = data_with_duplicates
            .iter()
            .combinations(2)
            .filter(|v| v[0].0 != v[1].0)
            .flat_map(|v| {
                let (a_key, a_value) = v[0];
                let (b_key, b_value) = v[1];
                if Arc::ptr_eq(a_value, b_value) {
                    println!("{} points to the same Arc as {}", a_key, b_key);
                    vec![
                        (a_key.clone(), a_key.clone()),
                        (b_key.clone(), a_key.clone()),
                    ]
                } else {
                    vec![(a_key.clone(), a_key.clone())]
                }
            })
            .collect();
        let data = port_matches
            .keys()
            .map(|k| (k.clone(), Arc::clone(data_with_duplicates[k])))
            .collect();

        (port_matches, data)
    }
    /// get a list of node indices
    pub fn nodes_ref(&self) -> Vec<NodeIndex> {
        self.nodes.keys().copied().collect()
    }

    ///Set the node value of an existing node
    pub fn set_node_data(
        &mut self,
        nx: NodeIndex,
        value: NodeData, //GenGraphNode<NodeData, PortType, WireData>,
    ) {
        *self.nodes.get_mut(&nx).unwrap() = value;
    }

    pub fn update_wire_data(&mut self, nx: NodeIndex, outputs: StableMap<PortName, WireData>) {
        for (port_name, wire_data) in outputs.into_iter() {
            self.wire_data
                .insert((nx, port_name), Arc::new(wire_data.into()));
        }
    }

    pub fn get_wire_data(
        &self,
        nx: &NodeIndex,
        port_name: &str,
    ) -> Option<&WireDataContainer<WireData>> {
        self.wire_data.get(&(*nx, port_name.into()))
    }

    /// create a connection between two port references
    pub fn add_edge_from_ref(&mut self, from: &PortRef, to: &PortRef) {
        assert!(from.io == IO::Out);
        assert!(to.io == IO::In);
        self.connect((from.node, from.name.clone()), (to.node, to.name.clone()));
    }
    /// create a connection between two ports
    pub fn connect(
        &mut self,
        from: (NodeIndex, impl Into<PortName>),
        to: (NodeIndex, impl Into<PortName>),
    ) {
        let from = PortRef {
            node: from.0,
            name: from.1.into(),
            io: IO::Out,
        };
        let to = PortRef {
            node: to.0,
            name: to.1.into(),
            io: IO::In,
        };

        //TODO: check for compatiablity, or if the edge alread exists
        // warn if already exists, panic/return result if incompatabible
        self.edges.push((from, to));
    }

    // remove any edges associated with the given port
    pub fn remove_edge(&mut self, port: &PortRef) {
        self.edges.retain(|(from, to)| port != from && port != to)
    }

    pub fn get_parent(&self, nx: &NodeIndex, in_port: PortName) -> Option<PortRef> {
        self.edges
            .iter()
            .find(|(_from, to)| to.node == *nx && to.name == in_port)
            .map(|(from, _to)| from.clone())
    }

    /// find the index of the port based on the order defined in the `GraphNode`
    /// panics if `port` is not valid
    pub fn port_index(&self, port: &PortRef) -> usize {
        match port.io {
            IO::In => self
                .get_node(port.node)
                .inputs()
                .iter()
                .position(|n| *n.0 == *port.name)
                .unwrap_or_else(|| {
                    panic!("PortId must have valid input node index and port id {port:?}",)
                }),
            IO::Out => self
                .get_node(port.node)
                .outputs()
                .iter()
                .position(|n| *n.0 == *port.name)
                .unwrap_or_else(|| {
                    panic!("PortId must have valid input node index and port id {port:?}",)
                }),
        }
    }
    /// Find a nodes direct parents and the associated labels
    pub fn incoming_edges(&self, nx: &NodeIndex) -> Vec<(PortRef, PortRef)> {
        self.edges
            .iter()
            .filter_map(|(from, to)| {
                if to.node == *nx {
                    Some((from.clone(), to.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Find the edges that that originate at `nx`
    pub fn outgoing_edges(&self, nx: &NodeIndex) -> Vec<PortRef> {
        self.edges
            .iter()
            .filter_map(|(from, to)| {
                if from.node == *nx {
                    Some(to.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Topological sort using Kahn's algorithm
    /// returns a list of NodeIndices
    pub fn topological_sort(&self) -> Vec<NodeIndex> {
        let mut sorted = vec![];
        let mut working_edges = self.edges.clone();

        let mut no_incoming: Vec<_> = self
            .nodes
            .keys()
            .filter(|nx| !Self::has_incoming(nx, &working_edges))
            .copied()
            .collect();

        while let Some(nx) = no_incoming.pop() {
            sorted.push(nx);
            while let Some(ex) = Self::next_connected_edge(&nx, &working_edges) {
                let edge = working_edges.swap_remove(ex);
                let mx = edge.1.node;
                if !Self::has_incoming(&mx, &working_edges) {
                    no_incoming.push(mx);
                }
            }
        }
        if working_edges.is_empty() {
            sorted
        } else {
            panic!("graph has cycles!")
        }
    }

    ///// Execute network using topological sort, starting from `start`
    ///// and only processing decendents
    ///// Caller is responsible for handling errors that occur during computation
    //pub fn exectute_sub_network(&mut self, start: NodeIndex) {
    //    let nodes: Vec<_> = self
    //        .topological_sort()
    //        .into_iter()
    //        .filter(|&nx| self.is_self_or_dependent(start, nx))
    //        .collect();
    //    nodes.iter().for_each(|nx| self.compute_node(nx))
    //}
    //
    ///// Execute network using topological sort
    ///// Caller is responsible for handling errors that occur during computation
    //pub fn execute_network(&mut self) {
    //    self.wire_data.clear();
    //    let mut ordered = self.topological_sort();
    //    ordered.iter_mut().for_each(|nx| self.compute_node(nx))
    //}

    // async compute

    //pub async fn execute_network_async(&mut self) {
    //    self.wire_data.clear();
    //    let mut ordered = self.topological_sort();
    //    ordered.iter_mut().for_each(|nx| self.compute_node(nx))
    //}

    // perform computation by supplying inputs
    //pub async fn clone_compute(
    //    &self,
    //    nx: NodeIndex,
    //) -> Option<(OrderMap<String, WireData>, NodeData)> {
    //    let node = self.get_node(nx).clone();
    //    let inputs = self.get_input_data(&nx);
    //    if inputs.len() == node.inputs().len() {
    //        Some(node.compute(inputs.into_iter().map(|(k, v)| (k, v.clone())).collect()))
    //    } else {
    //        None
    //    }
    //}
    //
    // perform computation by supplying inputs
    //pub fn compute_node(&mut self, nx: &NodeIndex) {
    //    let node = self.get_mut_node(*nx).clone();
    //    let inputs = self.get_input_data(nx);
    //    if inputs.len() == node.inputs().len() {
    //        let (outputs, node) =
    //            node.compute(inputs.into_iter().map(|(k, v)| (k, v.clone())).collect());
    //        self.set_node_data(*nx, node);
    //        self.update_wire_data(*nx, outputs);
    //    }
    //}

    //fn is_self_or_dependent(&self, root: NodeIndex, to_check: NodeIndex) -> bool {
    //    if root == to_check {
    //        true
    //    } else {
    //        self.incoming_edges(&to_check)
    //            .into_iter()
    //            .any(|(from, _to)| self.is_self_or_dependent(root, from.node))
    //    }
    //}

    /// Determine if a node has any incoming connections
    fn has_incoming(nx: &NodeIndex, edges: &[Edge]) -> bool {
        edges.iter().any(|(_from, to)| to.node == *nx)
    }

    /// Find the index of `edges` corresponding to the first
    /// connection starting from `nx` (if it exists)
    fn next_connected_edge(nx: &NodeIndex, edges: &[Edge]) -> Option<usize> {
        edges.iter().position(|(from, _to)| from.node == *nx)
    }

    //pub(crate) fn get_compute(
    //    &self,
    //    nx: u32,
    //) -> Option<impl AsyncFn() -> > {
    //    let node = self.get_node(nx).clone();
    //    let inputs = self.get_input_data(&nx);
    //    let inputs: OrderMap<String, Arc<Mutex<WireData>>> =
    //        inputs.into_iter().map(|(a, b)| (a, b.clone())).collect();
    //    if inputs.len() == node.inputs().len() {
    //        Some(async move || {
    //            let (new_node, output) = node.compute(inputs);
    //            (new_node, output)
    //        })
    //    } else {
    //        None
    //    }
    //}
    pub fn get_compute(
        &self,
        nx: NodeIndex,
    ) -> (NodeData, StableMap<String, WireDataContainer<WireData>>) {
        let node = self.get_node(nx);
        let wire_data = self.get_input_data(&nx);
        (node.clone(), wire_data)
    }
    #[allow(clippy::type_complexity)]
    pub fn compute_node(
        nx: NodeIndex,
        node: NodeData,
        input_guarded: StableMap<String, WireDataContainer<WireData>>,
    ) -> (
        u32,
        Result<(StableMap<String, WireData>, NodeData), NodeError>,
    ) {
        let output = { node.compute(input_guarded) };

        (nx, output)
    }
    pub async fn async_compute(
        nx: NodeIndex,
        node: NodeData,
        input_guarded: StableMap<String, WireDataContainer<WireData>>,
    ) -> (
        u32,
        Result<(StableMap<String, WireData>, NodeData), NodeError>,
    ) {
        Self::compute_node(nx, node, input_guarded)
    }

    /// get all nodes that have no parents
    pub fn get_roots(&self) -> Vec<NodeIndex> {
        self.nodes
            .keys()
            .filter(|nx| self.incoming_edges(nx).is_empty())
            .copied()
            .collect()
    }
}

impl<NodeData, PortType, WireData> Default for Graph<NodeData, PortType, WireData>
where
    NodeData: GraphNode<NodeData, PortType, WireData> + Clone,
    PortType: Clone + Default,
    WireData: std::fmt::Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[derive(Clone, Debug)]
    struct IdentityNode {}
    #[derive(Clone, Debug)]
    struct ConstantNode {
        value: u32,
    }

    #[derive(Clone, Debug)]
    enum Node {
        Identity(IdentityNode),
        Constant(ConstantNode),
    }

    impl GraphNode<Node, (), u32> for Node {
        fn inputs(&self) -> StableMap<String, ()> {
            match self {
                Node::Identity(_node) => [("in".to_string(), ())].into(),
                Node::Constant(_node) => [].into(),
            }
        }

        fn outputs(&self) -> StableMap<String, ()> {
            match self {
                Node::Identity(_node) => [("out".to_string(), ())].into(),
                Node::Constant(_node) => [("out".to_string(), ())].into(),
            }
        }

        fn compute(
            self,
            inputs: StableMap<String, WireDataContainer<u32>>,
        ) -> Result<(StableMap<String, u32>, Node), NodeError> {
            Ok(match &self {
                Node::Identity(_node) => (
                    [("out".to_string(), *inputs["in"].read().unwrap())].into(),
                    self,
                ),
                Node::Constant(node) => ([("out".to_string(), node.value)].into(), self),
            })
        }
    }

    #[test]
    fn sort() {
        let mut g: Graph<Node, (), u32> = Graph::new();

        let n8 = g.node(Node::Identity(IdentityNode {}));
        let n7 = g.node(Node::Identity(IdentityNode {}));
        let n6 = g.node(Node::Identity(IdentityNode {}));
        let n5 = g.node(Node::Identity(IdentityNode {}));
        let n4 = g.node(Node::Identity(IdentityNode {}));
        let n3 = g.node(Node::Identity(IdentityNode {}));
        let n2 = g.node(Node::Identity(IdentityNode {}));
        let n1 = g.node(Node::Identity(IdentityNode {}));

        g.connect((n1, "out"), (n3, "in"));
        g.connect((n1, "out"), (n2, "in"));
        g.connect((n3, "out"), (n4, "in"));
        g.connect((n4, "out"), (n5, "in"));
        g.connect((n5, "out"), (n6, "in"));
        g.connect((n6, "out"), (n7, "in"));
        g.connect((n7, "out"), (n8, "in"));
        assert_eq!(g.topological_sort(), vec![7, 6, 5, 4, 3, 2, 1, 0]);
    }

    #[test]
    fn process() {
        let mut g: Graph<Node, (), u32> = Graph::new();

        let n1 = g.node(Node::Constant(ConstantNode { value: 7 }));
        let n2 = g.node(Node::Identity(IdentityNode {}));
        let n3 = g.node(Node::Identity(IdentityNode {}));
        let n4 = g.node(Node::Identity(IdentityNode {}));

        // leave a node unconnected to check that it doesn't get a value propogated
        let n_unconnected = g.node(Node::Identity(IdentityNode {}));

        g.connect((n1, "out"), (n3, "in"));
        g.connect((n1, "out"), (n2, "in"));
        g.connect((n3, "out"), (n4, "in"));

        //Propogate values
        for nx in g.topological_sort() {
            let (node, input_guarded) = g.get_compute(nx);
            let (_, output) = Graph::compute_node(nx, node, input_guarded);
            g.update_wire_data(nx, output.unwrap().0);
        }

        assert_eq!(*g.get_wire_data(&n1, "out").unwrap().read().unwrap(), 7);
        assert_eq!(*g.get_wire_data(&n2, "out").unwrap().read().unwrap(), 7);
        assert_eq!(*g.get_wire_data(&n3, "out").unwrap().read().unwrap(), 7);
        assert!(g.get_wire_data(&n_unconnected, "out").is_none());
    }
}
