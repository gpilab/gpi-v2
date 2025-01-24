use iced::{widget::text, Element};

use crate::{
    app::{Message, PortDataContainer},
    interface::node::default_node_size,
    OrderMap,
};

pub trait GUINode: derive_more::Debug {
    //TODO make this more understandable. clearer distinction between graph and gui?
    // split out port names, and compute logic?
    //fn network_node(&self) -> GraphNode<PortType, PortData>;

    //TODO: Port validation logic? here, or handled at the portType level?
    //TODO: conversion logic?

    fn name(&self) -> String;

    fn view<'a>(
        &'a self,
        _id: u32,
        _input_data: OrderMap<String, &PortDataContainer>,
    ) -> (iced::Size, Element<'a, Message>) {
        (default_node_size(), text("default").into())
    }

    fn config_view<'a>(
        &'a self,
        _id: u32,
        _input_data: OrderMap<String, &PortDataContainer>,
    ) -> Option<Element<'a, Message>> {
        None
    }
}
