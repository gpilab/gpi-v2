use std::fmt::Debug;

use petgraph::{
    graph::{DiGraph, EdgeIndex},
    visit::{Topo, Walker},
};
use pyo3::Python;
use serde::Serialize;

use crate::{
    node::Node,
    node_type::NodeType,
    port::{NodeIndex, Port, PortName, PortType},
};

#[derive(Debug)]
pub struct Network<'a> {
    pub(crate) g: DiGraph<Node<'a>, ()>,
}

impl<'a> Default for Network<'a> {
    ///Initialize an empty network
    fn default() -> Self {
        Network {
            g: DiGraph::<Node, ()>::new(),
        }
    }
}

/// A network of nodes. Nodes are connected via ports.
impl Network<'_> {
    //// Mutators

    /// Add a new node to the network
    pub fn add_node(
        &mut self,
        n_type: NodeType,
        input: Vec<(PortName, PortType)>,
        output: Vec<(PortName, PortType)>,
    ) -> NodeIndex {
        let node_id = self.g.add_node(Node::new(n_type, input, output));

        // now that we have a node_id, set it on the node
        self.g.node_weight_mut(node_id).unwrap().node_id = node_id;
        node_id
    }

    /// connect an input node to an output node
    pub fn connect_nodes<T: Into<PortName> + Clone>(
        &mut self,
        from_node_idx: NodeIndex,
        from_port_name: T,
        to_node_idx: NodeIndex,
        to_port_name: T,
    ) -> EdgeIndex {
        //// Check Port Compatability
        {
            // need to use immutable references to check two nodes at once
            let from_node = self.g.node_weight(from_node_idx).unwrap();
            let to_node = self.g.node_weight(to_node_idx).unwrap();
            // TODO: Custom error types and returning a result would be nice here
            assert!(
                from_node.can_connect_child(from_port_name.clone(), to_node, to_port_name.clone()),
                "Tried to connect incompatible ports: {from_node:?}->{to_node:?}"
            );
        }
        //// Update to_node's input to point to from_node
        let to_node = self.g.node_weight_mut(to_node_idx).unwrap();
        to_node.connect_input(to_port_name.into(), from_node_idx, from_port_name.into());

        //// Create the edge in the graph as well
        self.g.add_edge(from_node_idx, to_node_idx, ())
    }

    /// Loop through the graph and propogate values
    pub fn process(&mut self, py: Python) {
        let mut topo = Topo::new(&self.g);

        while let Some(nx) = topo.next(&self.g) {
            let node = self.g.node_weight(nx).unwrap();

            node.node_type.clone().compute(nx, self, py);
        }
    }
    pub fn display_final_node(&self) {
        let topo = Topo::new(&self.g);
        let nx = topo.iter(&self.g).last().unwrap();
        let node = self.g.node_weight(nx).unwrap();
        println!("{}", node.get_output_data(&"out".into()))
    }

    //// Accessors

    #[must_use]
    pub fn get_output_data(&self, port_id: (NodeIndex, PortName)) -> &Port {
        self.g
            .node_weight(port_id.0)
            .unwrap()
            .get_output_data(&port_id.1)
    }

    pub(crate) fn retrieve_input_data(&self, node: &Node, input_port_name: &PortName) -> &Port {
        let parent_port_id = node.get_connected_port_id(input_port_name);
        self.get_output_data(parent_port_id)
    }
}
