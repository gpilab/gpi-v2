use canvas::{Path, Stroke};
use gpi_iced::graph::Graph;
use gpi_iced::widget::shapes::ShapeId;
use gpi_iced::widget::workspace::{self, workspace};
use iced::border::{radius, rounded};
use iced::widget::{column, *};
use iced::Element;
use iced::*;
use ndarray::{ArrayD, ArrayViewD, IxDyn};
const NODE_WIDTH: f32 = 100.;
const NODE_HEIGHT: f32 = 60.;

pub fn main() -> iced::Result {
    #[cfg(target_arch = "wasm32")]
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    application("gpi_v2", Example::update, Example::view)
        .antialiasing(true)
        .theme(theme)
        .window_size((800., 600.))
        .decorations(true)
        .run()
}

fn theme(_state: &Example) -> Theme {
    Theme::Ferra
}

#[derive(Debug, Clone)]
enum PortData {
    Integer(Option<ArrayD<i64>>),
    Real(Option<ArrayD<f64>>),
    Complex(Option<ArrayD<(f64, f64)>>),
}

#[derive(Default, Debug)]
struct Node {
    name: String,
    value: u32,
    inputs: Vec<(String, PortData)>,
    outputs: Vec<(String, PortData)>,
}

struct Example {
    graph: Graph<Node, String>,
    shapes: workspace::State<()>,
    selected_shape: Option<ShapeId>,
    config: f32,
    theme: Theme,
}

impl Default for Example {
    fn default() -> Example {
        let points = [
            Point::new(300., 100.),
            Point::new(300., 200.),
            Point::new(150., 300.),
            Point::new(400., 300.),
            Point::new(100., 400.),
            Point::new(250., 400.),
        ];

        let initial_nodes = vec![
            Node {
                name: "a".into(),
                value: 0,
                inputs: vec![],
                outputs: vec![(
                    "out".into(),
                    PortData::Real(Some(ArrayD::from_shape_fn(IxDyn(&[16]), |_i| 0.))),
                )],
            },
            Node {
                name: "b".into(),
                value: 0,
                inputs: vec![("in".into(), PortData::Real(None))],
                outputs: vec![("out".into(), PortData::Real(None))],
            },
            Node {
                name: "c".into(),
                value: 0,
                inputs: vec![("in".into(), PortData::Real(None))],
                outputs: vec![("out".into(), PortData::Real(None))],
            },
            Node {
                name: "d".into(),
                value: 0,
                inputs: vec![("in".into(), PortData::Real(None))],
                outputs: vec![("out".into(), PortData::Real(None))],
            },
            Node {
                name: "e".into(),
                value: 0,
                inputs: vec![("in".into(), PortData::Real(None))],
                outputs: vec![("out".into(), PortData::Real(None))],
            },
            Node {
                name: "f".into(),
                value: 0,
                inputs: vec![("in".into(), PortData::Real(None))],
                outputs: vec![("out".into(), PortData::Real(None))],
            },
        ];

        let mut g = Graph::new();
        initial_nodes.into_iter().for_each(|n| {
            g.add_node(n);
        });
        g.add_edge((0, "out".into()), (1, "in".into()));
        g.add_edge((1, "out".into()), (2, "in".into()));
        g.add_edge((1, "out2".into()), (3, "in".into()));
        g.add_edge((2, "out".into()), (4, "in".into()));
        g.add_edge((3, "out".into()), (5, "in".into()));
        let nodes_refs = g.nodes_ref();
        let nr: Vec<_> = nodes_refs
            .iter()
            .zip(points.iter())
            .map(|((k, _v), p)| (*k, (), *p))
            .collect();

        Self {
            graph: g,
            shapes: workspace::State::<()>::new(nr),
            selected_shape: None,
            config: 50.,
            theme: Theme::Ferra,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Message {
    OnDrag(ShapeId, Point),
    Pan(Vector),
    Config(f32),
    OnSelect(ShapeId),
}

impl Example {
    fn update(&mut self, message: Message) {
        match message {
            Message::Pan(delta) => {
                self.shapes.camera.position.x -= delta.x * 2.;
                self.shapes.camera.position.y -= delta.y * 2.;
            }
            Message::Config(v) => self.config = v,
            Message::OnSelect(shape_id) => {
                self.selected_shape = Some(shape_id);

                let node = self.graph.get_mut_node(shape_id);
                node.value += 1;

                let mut ordered = self.graph.sorted_subset(shape_id);

                ordered.iter_mut().for_each(|nx| {
                    let parent_sum: u32 = self
                        .graph
                        .incoming_edges(nx)
                        .into_iter()
                        .map(|(from, _to)| self.graph.get_node(from.0).value)
                        .sum();

                    let parent_array = self
                        .graph
                        .incoming_edges(nx)
                        .into_iter()
                        .map(|(from, _to)| &self.graph.get_node(from.0).outputs[0].1)
                        .next()
                        .unwrap_or(&PortData::Real(None));

                    let new_array = parent_array.clone();

                    let node = self.graph.get_mut_node(*nx);
                    node.value += parent_sum;
                    if let PortData::Real(Some(array)) = &mut node.outputs[0].1 {
                        array[0] = node.value as f64;
                    } else {
                        node.outputs[0].1 = new_array;
                    }

                    dbg!(node);
                });
            }
            Message::OnDrag(shape_index, cursor_position) => {
                self.shapes
                    .shapes
                    .0
                    .get_mut(&shape_index)
                    .expect("Shape index must exist")
                    .position = cursor_position
            }
        };
    }

    fn view(&self) -> Element<Message, Theme, Renderer> {
        const SEPERATOR: f32 = 1.0;

        let button_style = |t: &Theme, s| {
            let mut style = button::secondary(t, s);
            style.border.radius = radius(0.);
            style
        };

        let file_commands = row![
            horizontal_space(),
            button(text("New").line_height(0.6))
                .on_press(Message::Config(20.))
                .style(button_style),
            horizontal_space(),
            button(text("Load").line_height(0.6))
                .on_press(Message::Config(40.))
                .style(button_style),
            horizontal_space(),
            button(text("Save").line_height(0.6))
                .on_press(Message::Config(60.))
                .style(button_style),
            horizontal_space(),
        ]
        .spacing(2.0)
        .padding([5., 10.]);

        let config = column![
            vertical_space().height(20.),
            row!["Label1", slider(0.0..=100.0, self.config, Message::Config)]
                .spacing(20.)
                .align_y(Alignment::Center),
            row!["Label2", slider(0.0..=100.0, self.config, Message::Config)]
                .spacing(20.)
                .align_y(Alignment::Center),
            row!["Label3", slider(0.0..=100.0, self.config, Message::Config)]
                .spacing(20.)
                .align_y(Alignment::Center),
            vertical_space(),
        ]
        .spacing(5.0)
        .padding(5.);

        let workspace = workspace(
            &self.shapes,
            |id, _nx| {
                let node = self.graph.get_node(id);
                let is_selected = match self.selected_shape {
                    Some(s_id) => id == s_id,
                    None => false,
                };

                let name = node.name.clone();
                let value = node.value;

                let content =
                    column![text(name), text(value).style(text::secondary)].align_x(Center);

                //let output_port_pos = self
                //    .graph
                //    .outgoing_edges(&id)
                //    .iter()
                //    .enumerate()
                //    .map(|(i, (_from, _to))| {
                //        let port_x = (i + 1) as f32 * (NODE_WIDTH / 8.);
                //        Vector::new(port_x, NODE_HEIGHT)
                //    })
                //    .map(pin(button(())));

                container(content)
                    .center(Length::Fill)
                    .width(NODE_WIDTH)
                    .height(NODE_HEIGHT)
                    .style(move |t: &Theme| {
                        let outline_color = match is_selected {
                            true => t.extended_palette().primary.strong.color,
                            false => t.extended_palette().secondary.strong.color,
                        };
                        container::transparent(t)
                            .border(rounded(5.).color(outline_color).width(2.))
                            .background(self.theme.palette().background)
                    })
                    .into()
            },
            |id, _nx, points| {
                let edges = self.graph.outgoing_edges(&id);
                edges
                    .iter()
                    .enumerate()
                    .map(|(i, (from, to))| {
                        let port_x = (i + 1) as f32 * (NODE_WIDTH / 8.);
                        (
                            points[&from.0] + Vector::new(port_x, NODE_HEIGHT),
                            points[&to.0] + Vector::new(NODE_WIDTH / 8., 0.),
                        )
                    })
                    .map(|(from, to)| {
                        (
                            Path::new(|builder| {
                                builder.move_to(from);
                                let mid = f32::abs((from.y - to.y) * 0.5).max(15.0);
                                builder.bezier_curve_to(
                                    (from.x, from.y + mid).into(),
                                    (to.x, to.y - mid).into(),
                                    to,
                                );
                            }),
                            Stroke::default()
                                .with_width(3.0)
                                .with_color(self.theme.extended_palette().secondary.weak.color)
                                .with_line_cap(canvas::LineCap::Round),
                        )
                    })
                    .collect()
            },
        )
        .on_shape_drag(Message::OnDrag)
        .on_press(Message::OnSelect)
        .pan(Message::Pan);

        ////View
        row![
            container(
                column![
                    //// File
                    file_commands.align_y(Alignment::Center).width(Length::Fill),
                    ////Config
                    horizontal_rule(SEPERATOR),
                    ////
                    config
                ]
                .height(Length::Fill)
                .width(250.),
            ),
            vertical_rule(SEPERATOR),
            container(workspace)
                .height(Length::Fill)
                .width(Length::Fill)
        ]
        .into()
    }
}
