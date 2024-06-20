use std::collections::HashMap;
use Vec;

use anyhow;
use dagre_rust::{layout, GraphConfig, GraphEdge, GraphNode};
use eframe::egui;
use eframe::epaint::Color32;
use egui::emath::Rect;
use egui::{FontId, Pos2, Rounding, Stroke, Vec2};
use graphlib_rust::{Graph, GraphOption};

/// The `DagNode` struct represents a node in a directed acyclic graph (DAG).
/// Each `DagNode` has a name and a list of parent nodes.

#[derive(Clone, Debug)]
pub struct DagNode {
    block_hash: String,
    block_number: Option<u64>,
    timestamp: Option<u64>,
    block_creator: Option<String>,
    parent_nodes: Vec<String>,
}

impl DagNode {
    pub fn new(
        name: &str,
        parent_node: &Vec<String>,
        block_number: Option<u64>,
        timestamp: Option<u64>,
        block_creator: Option<String>,
    ) -> Self {
        Self {
            block_hash: name.to_string(),
            block_number,
            timestamp,
            block_creator,
            parent_nodes: parent_node.clone(),
        }
    }
}

pub(crate) struct DagViewer {
    dag_graph: Graph<GraphConfig, GraphNode, GraphEdge>,
    dag_node_datas: HashMap<String, DagNode>,
    scale: f32,
    horizontal_scroll: f32,
    vertical_scroll: f32,
    node_radius_size: f32,
    // edge_interval_pixel: f64,
    // indent: Pos2,
    font: FontId,
    invalid_layout: bool,
    selected_node: Option<String>,
}

impl DagViewer {
    pub(crate) fn new(
        dag_nodes: Vec<DagNode>,
        _edge_interval_pixel: f64,
        node_radius_size: f32,
        _indent_x: f32,
        _indent_y: f32,
    ) -> Self {
        let mut option = GraphOption::default();
        option.compound = Some(true);
        option.directed = Some(true);

        let mut dag_graph = Graph::<GraphConfig, GraphNode, GraphEdge>::new(Some(option));
        dag_graph.graph_mut().rankdir = Some("lr".to_string());
        dag_graph.graph_mut().nodesep = Some(200.0);
        dag_graph.graph_mut().marginx = Some(50.0);

        let mut dag_node_datas = HashMap::<String, DagNode>::new();
        dag_nodes.iter().for_each(|node| {
            dag_node_datas.insert(node.block_hash.clone(), node.clone());
        });
        Self::dagnodes_into_graph(&mut dag_graph, dag_nodes).expect("Add not failed");

        Self {
            dag_graph,
            scale: 1.0,
            vertical_scroll: 0.0,
            horizontal_scroll: 0.0,
            //edge_interval_pixel,
            node_radius_size,
            // indent: Pos2::new(indent_x, indent_y),
            font: FontId::monospace(16.0),
            invalid_layout: false,
            selected_node: None,
            dag_node_datas,
        }
    }

    /// Convert DAG nodes to viewer nodes
    fn dagnodes_into_graph(
        graph: &mut Graph<GraphConfig, GraphNode, GraphEdge>,
        dag_nodes: Vec<DagNode>,
    ) -> anyhow::Result<()> {
        if dag_nodes.is_empty() {
            return Ok(());
        };

        //  Put all node into graph
        for dag_node in &dag_nodes {
            let mut node = GraphNode::default();
            node.rank = Some(0);
            graph.set_node(dag_node.block_hash.clone(), Some(node));
        }

        // Set parent and edge
        for dag_node in &dag_nodes {
            for parent in &dag_node.parent_nodes {
                // graph
                //     .set_parent(&dag_node.name, Some(parent.clone()))
                //     .expect("Set parent failed");
                graph
                    .set_edge(&parent, &dag_node.block_hash, None, None)
                    .expect("Set edge failed");
            }
        }
        Ok(())
    }

    pub fn set_need_layout(&mut self) {
        self.invalid_layout = true;
    }

    fn layout(&mut self, ctx: &egui::Context) -> anyhow::Result<()> {
        if !self.invalid_layout {
            return Ok(());
        }
        self.calculate_node_size(ctx);
        layout::run_layout(&mut self.dag_graph);
        self.invalid_layout = false;
        Ok(())
    }

    fn calculate_node_size(&mut self, ctx: &egui::Context) {
        for node_key in self.dag_graph.nodes() {
            let node = self
                .dag_graph
                .node_mut(&node_key)
                .expect("Failed to find node");
            // let text_size = ctx
            //.fonts(|f| f.layout(node_key, self.font.clone(), Color32::GRAY, f32::INFINITY))
            // .as_ref()
            // .rect;
            let glyph_width = ctx.fonts(|f| f.glyph_width(&self.font, 'm'));
            let row_height = ctx.fonts(|f| f.row_height(&self.font));
            node.width = glyph_width * node_key.len() as f32;
            node.height = row_height;
        }
    }

    fn draw_nodes(&mut self, ui: &mut egui::Ui, cursor_pos: Pos2) {
        for node_key in self.dag_graph.nodes() {
            let node = self.dag_graph.node(&node_key).expect("Failed to find node");

            let node_pos = Pos2::new(
                node.x + self.horizontal_scroll,
                node.y + self.vertical_scroll,
            );
            // println!("draw_nodes | node_pos : {:?}", node_pos);
            // ui.painter()
            //     .circle_filled(node_pos, self.node_radius_size, Color32::LIGHT_BLUE);

            let fill_pos = Pos2::new(
                node_pos.x - node.width / 2.0,
                node_pos.y - node.height / 2.0,
            );
            let fill_rect = Rect::from_min_size(fill_pos, Vec2::new(node.width, node.height));
            let is_hovered = fill_rect.contains(cursor_pos);
            let is_clicked = is_hovered && ui.input(|i| i.pointer.any_click());

            let theme_text_color = ui.visuals().text_color();
            let (node_text_color, node_bg_color) = if is_hovered {
                (
                    Color32::from_rgb(
                        255 - theme_text_color.r(),
                        255 - theme_text_color.g(),
                        255 - theme_text_color.b(),
                    ),
                    theme_text_color,
                )
            } else {
                (theme_text_color, Color32::TRANSPARENT)
            };

            if is_clicked {
                self.selected_node = Some(node_key.clone());
            }

            let radius = 5.0;
            ui.painter()
                .rect_filled(fill_rect, Rounding::from(radius), node_bg_color);

            ui.painter().rect_stroke(
                fill_rect,
                Rounding::from(radius),
                Stroke::new(1.0, node_text_color),
            );

            ui.painter().text(
                node_pos,
                egui::Align2::CENTER_CENTER,
                &node_key,
                self.font.clone(),
                node_text_color,
            );
        }
    }

    fn draw_edges(&self, ui: &mut egui::Ui) {
        for e in &self.dag_graph.edges() {
            let graph_edge = self
                .dag_graph
                .edge(&e.v, &e.w, None)
                .expect("Can't find edge");
            // println!("edge:  {:?}", graph_edge);
            let points = graph_edge
                .points
                .as_ref()
                .clone()
                .expect("Can't find edge points");
            let from_pos = Pos2::new(
                points[0].x + self.horizontal_scroll,
                points[0].y + self.vertical_scroll,
            );
            let to_pos = Pos2::new(
                points[points.len() - 1].x + self.horizontal_scroll,
                points[points.len() - 1].y + self.vertical_scroll,
            );

            self.draw_arrow(ui, from_pos, to_pos);
        }
    }

    fn draw_arrow(&self, ui: &mut egui::Ui, start: Pos2, end: Pos2) {
        let theme_text_color = ui.visuals().text_color();

        // Calculate direction and normalize it
        let direction = Vec2::new(end.x - start.x, end.y - start.y);
        let length = (direction.x.powi(2) + direction.y.powi(2)).sqrt();
        let direction = direction / length;

        let start = start + direction * self.node_radius_size;
        let end = end - direction * self.node_radius_size;

        // // Draw the line segment
        ui.painter()
            .line_segment([start, end], (2.0, theme_text_color));

        // Arrowhead size
        let arrowhead_size = 5.0;

        // Calculate the points of the arrowhead
        let arrow_point1 = end - direction * arrowhead_size
            + Vec2::new(-direction.y, direction.x) * arrowhead_size * 0.5;
        let arrow_point2 = end - direction * arrowhead_size
            + Vec2::new(direction.y, -direction.x) * arrowhead_size * 0.5;

        // Draw the arrowhead
        ui.painter()
            .line_segment([end, arrow_point1], (2.0, theme_text_color));
        ui.painter()
            .line_segment([end, arrow_point2], (2.0, theme_text_color));
    }

    fn maybe_show_node_info(&mut self, ui: &mut egui::Ui) {
        if self.selected_node.is_none() {
            return;
        }

        let mut open = true;
        egui::Window::new("Node Information")
            .open(&mut open)
            .show(ui.ctx(), |ui| {
                let node_key = self.selected_node.as_ref().unwrap();
                // println!("Open node: {:?}", node_key);

                ui.label(format!("BlockHash: {}", node_key.as_str()));
                let node = self.dag_node_datas.get(node_key);
                if node.is_none() {
                    return;
                }

                let node = self.dag_node_datas.get(node_key).unwrap();

                node.block_number.map(|block_number| {
                    ui.label(format!("BlockNumber: {}", block_number));
                });

                node.timestamp.map(|timestamp| {
                    ui.label(format!("Timestamp: {}", timestamp));
                });

                node.block_creator.as_ref().map(|block_creator| {
                    ui.label(format!("Creator: {}", block_creator));
                });
            });

        if !open {
            self.selected_node = None;
        }
    }
}

impl eframe::App for DagViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.layout(ctx).expect("Auto layout failed");

            // Add a ScrollArea
            egui::ScrollArea::both().show(ui, |ui| {
                // Calculate the scroll delata for zooming
                let zoom_delta = if ui.rect_contains_pointer(ui.max_rect()) {
                    let scroll_delta = ctx.input(|i| i.raw_scroll_delta);
                    let zoom_delta = ctx.input(|i| i.zoom_delta());

                    self.horizontal_scroll += scroll_delta.x;
                    self.vertical_scroll += scroll_delta.y;

                    zoom_delta - 1.0
                } else {
                    0.0
                };

                // Adjust the scale
                self.scale += zoom_delta;

                // Maximum 5 times, minimum 0.5 times
                self.scale = self.scale.clamp(0.5, 5.0);

                // Apply scaling to UI
                ui.ctx().set_pixels_per_point(self.scale);

                // Draw edges and nodes
                self.draw_edges(ui);

                let pos = ctx
                    .input(|i| i.pointer.latest_pos())
                    .or_else(|| Some(Pos2::new(0.0, 0.0)))
                    .unwrap();
                self.draw_nodes(ui, pos);
            });
            self.maybe_show_node_info(ui);
        });
    }
}
