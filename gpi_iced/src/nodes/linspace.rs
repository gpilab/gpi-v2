use super::PortData;
use crate::{app::Message, math::linspace, node_data::NodeData};
use iced::{
    widget::{
        column, container, horizontal_rule, horizontal_space, row, text, text_input, TextInput,
    },
    Alignment::Center,
    Color, Element,
    Length::{Fill, Shrink},
};
use ordermap::OrderMap;
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct LinspaceConfig {
    start: f64,
    stop: f64,
    num: i64,
}

impl Default for LinspaceConfig {
    fn default() -> Self {
        Self {
            start: -100.,
            stop: 100.,
            num: 100,
        }
    }
}

impl LinspaceConfig {
    pub fn new(start: f64, stop: f64, num: i64) -> Self {
        Self { start, stop, num }
    }

    pub fn compute(
        &self,
        _inputs: OrderMap<String, &RefCell<PortData>>,
    ) -> OrderMap<String, PortData> {
        //node after potential modifications
        let LinspaceConfig { start, stop, num } = self;
        //let node: Self = *node.as_ref();
        let data: Vec<_> = linspace(*start as f32, *stop as f32, *num as i32);

        [(
            "out".into(),
            PortData::Real(
                data.clone()
                    .into_iter()
                    .map(|v| v as f64)
                    .collect::<Vec<_>>()
                    .into(),
            ),
        )]
        .into()
    }

    pub fn view(&self, id: u32) -> Element<Message> {
        fn numeric_input(input: TextInput<Message>) -> Element<Message> {
            column![
                input
                    .padding(0)
                    .style(|t, s| {
                        let d = text_input::default(t, s);
                        text_input::Style {
                            border: d.border.color(Color::TRANSPARENT),
                            background: iced::Background::Color(Color::TRANSPARENT),
                            ..d
                        }
                    })
                    .align_x(Center),
                container(horizontal_rule(0)).width(20.).height(1),
            ]
            .height(Shrink)
            .width(30.)
            .align_x(Center)
            .into()
        }
        let start_input = numeric_input(text_input("0", &self.start.to_string()).on_input(
            move |value| {
                Message::UpdateNodeData(
                    id,
                    NodeData::Linspace(LinspaceConfig {
                        start: value.parse().unwrap_or(0.),
                        ..self.clone()
                    }),
                )
            },
        ));
        let stop_input = numeric_input(text_input("10", &self.stop.to_string()).on_input(
            move |value| {
                Message::UpdateNodeData(
                    id,
                    NodeData::Linspace(LinspaceConfig {
                        stop: value.parse().unwrap_or(0.),
                        ..self.clone()
                    }),
                )
            },
        ));
        let num_input = numeric_input(text_input("100", &self.num.to_string()).on_input(
            move |value| {
                Message::UpdateNodeData(
                    id,
                    NodeData::Linspace(LinspaceConfig {
                        num: value.parse().unwrap_or(0),
                        ..self.clone()
                    }),
                )
            },
        ));

        column![
            row![
                horizontal_space(),
                start_input,
                text(".."),
                stop_input,
                horizontal_space()
            ]
            .width(Fill)
            .align_y(Center)
            .padding(5.)
            .spacing(2.),
            row![horizontal_space(), text("#"), num_input, horizontal_space()].align_y(Center)
        ]
        .into()
    }
}