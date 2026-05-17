pub mod curve;
pub mod ellipse;
pub mod lighting;
pub mod line;
pub mod polygon;
pub mod rect;

use egui::Color32;

use crate::{
    canvas::Canvas,
    i18n::{Language, LanguageText},
    tools::Tool,
};

#[macro_export]
macro_rules! ordered_pair {
    ($a:expr, $b:expr) => {{
        let a = $a;
        let b = $b;
        if a <= b { (a, b) } else { (b, a) }
    }};
}

#[macro_export]
macro_rules! callout_tail {
    ($start:expr, $end:expr) => {{
        let start = $start;
        let end = $end;
        let (left, right) = $crate::ordered_pair!(start.0, end.0);
        let (_, bottom) = $crate::ordered_pair!(start.1, end.1);
        let w = (right - left).max(1);
        vec![
            (left + w / 3, bottom),
            (left + w / 5, bottom + w / 5),
            (left + w / 2, bottom),
        ]
    }};
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ShapeKind {
    Line,
    Curve,
    Ellipse,
    Rectangle,
    RoundedRectangle,
    Polygon,
    Triangle,
    RightTriangle,
    Diamond,
    Pentagon,
    Hexagon,
    RightArrow,
    LeftArrow,
    UpArrow,
    DownArrow,
    FourPointStar,
    FivePointStar,
    SixPointStar,
    RectCallout,
    EllipseCallout,
    Lightning,

    Extra,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeMode {
    Outline,
    Filled,
    FilledOutline,
}

impl ShapeMode {
    pub const ALL: [ShapeMode; 3] = [
        ShapeMode::Outline,
        ShapeMode::Filled,
        ShapeMode::FilledOutline,
    ];

    pub fn get_label(&self) -> LanguageText {
        match self {
            ShapeMode::Filled => LanguageText::Filled,
            ShapeMode::FilledOutline => LanguageText::FilledOutline,
            ShapeMode::Outline => LanguageText::Outline,
        }
    }
}

pub trait Shape {
    fn shape_id(&self) -> &'static str {
        match self.get_shape_kind() {
            ShapeKind::Line => "shape.line",
            ShapeKind::Curve => "shape.curve",
            ShapeKind::Ellipse => "shape.ellipse",
            ShapeKind::Rectangle => "shape.rectangle",
            ShapeKind::RoundedRectangle => "shape.rounded_rectangle",
            ShapeKind::Polygon => "shape.polygon",
            ShapeKind::Triangle => "shape.triangle",
            ShapeKind::RightTriangle => "shape.right_triangle",
            ShapeKind::Diamond => "shape.diamond",
            ShapeKind::Pentagon => "shape.pentagon",
            ShapeKind::Hexagon => "shape.hexagon",
            ShapeKind::RightArrow => "shape.right_arrow",
            ShapeKind::LeftArrow => "shape.left_arrow",
            ShapeKind::UpArrow => "shape.up_arrow",
            ShapeKind::DownArrow => "shape.down_arrow",
            ShapeKind::FourPointStar => "shape.four_point_star",
            ShapeKind::FivePointStar => "shape.five_point_star",
            ShapeKind::SixPointStar => "shape.six_point_star",
            ShapeKind::RectCallout => "shape.rect_callout",
            ShapeKind::EllipseCallout => "shape.ellipse_callout",
            ShapeKind::Lightning => "shape.lightning",
            ShapeKind::Extra => "shape.extra",
        }
    }

    fn get_shape_kind(&self) -> ShapeKind;

    fn get_shape_label(&self, current_language: &Language) -> LanguageText;

    fn draw(
        &mut self,
        canvas: &mut Canvas,
        start: (i32, i32),
        end: (i32, i32),
        outline: Color32,
        fill: Color32,
        thickness: i32,
        mode: ShapeMode,
    );

    fn paint_icon(&self, ui: &egui::Ui, rect: egui::Rect, selected: bool) {
        let color = if selected {
            egui::Color32::from_rgb(0, 95, 184)
        } else {
            egui::Color32::BLACK
        };
        paint_shape_icon(ui.painter(), rect, self.get_shape_kind(), color);
    }

    fn shape_button(
        &mut self,
        ui: &mut egui::Ui,
        current_language: &Language,
        selected: bool,
    ) -> egui::Response {
        let label = current_language.get_text(self.get_shape_label(current_language));
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::splat(22.0), egui::Sense::click());

        let bg = if selected {
            egui::Color32::from_rgb(218, 235, 252)
        } else if response.hovered() {
            egui::Color32::from_rgb(244, 248, 252)
        } else {
            egui::Color32::WHITE
        };
        let stroke = if selected {
            egui::Stroke::new(1.2, egui::Color32::from_rgb(0, 95, 184))
        } else {
            egui::Stroke::new(0.5, egui::Color32::from_rgb(226, 230, 236))
        };

        ui.painter().rect_filled(rect, 2.0, bg);
        ui.painter().rect_stroke(rect, 2.0, stroke);
        self.paint_icon(ui, rect.shrink(4.0), selected);

        response.on_hover_text(label)
    }
}

pub struct ShapeGroup {
    shapes: Vec<Box<dyn Shape>>,
    active: Option<usize>,
}

impl ShapeGroup {
    pub fn new() -> Self {
        let mut shapes: Vec<Box<dyn Shape>> = Vec::new();
        shapes.push(Box::new(line::LineShape::new()));
        shapes.push(Box::new(rect::RectShape::new()));
        shapes.push(Box::new(rect::RoundedRectShape::new()));
        shapes.push(Box::new(ellipse::EllipseShape::new()));
        shapes.push(Box::new(curve::CurveShape::new()));
        shapes.push(Box::new(polygon::PolygonShape::new(ShapeKind::Polygon)));
        shapes.push(Box::new(polygon::PolygonShape::new(ShapeKind::Triangle)));
        shapes.push(Box::new(polygon::PolygonShape::new(ShapeKind::Diamond)));
        shapes.push(Box::new(polygon::PolygonShape::new(ShapeKind::Pentagon)));
        shapes.push(Box::new(polygon::PolygonShape::new(ShapeKind::Hexagon)));
        shapes.push(Box::new(rect::RectCallout::new()));
        shapes.push(Box::new(ellipse::EllipseCallout::new()));
        shapes.push(Box::new(lighting::LightingShape::new()));

        Self {
            shapes,
            active: None,
        }
    }

    pub fn load_shape(&mut self, shape: Box<dyn Shape>) {
        self.shapes.push(shape);
    }

    pub fn remove_shape(&mut self, index: usize) {
        if index >= self.shapes.len() {
            return;
        }
        self.shapes.remove(index);
        self.active = match self.active {
            Some(active) if active == index => None,
            Some(active) if active > index => Some(active - 1),
            other => other,
        };
    }

    pub fn shapes(&self) -> &[Box<dyn Shape>] {
        &self.shapes
    }

    pub fn shapes_mut(&mut self) -> &mut [Box<dyn Shape>] {
        &mut self.shapes
    }

    pub fn active_index(&self) -> Option<usize> {
        self.active
    }

    pub fn active_shape(&self) -> Option<&(dyn Shape + '_)> {
        self.active
            .and_then(|active| self.shapes.get(active).map(Box::as_ref))
    }

    pub fn active_shape_mut(&mut self) -> Option<&mut (dyn Shape + '_)> {
        if let Some(active) = self.active {
            if let Some(shape) = self.shapes.get_mut(active) {
                Some(shape.as_mut())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn select(&mut self, index: usize) {
        if index < self.shapes.len() {
            self.active = Some(index);
        }
    }

    #[allow(dead_code)]
    pub fn select_first(&mut self) {
        if !self.shapes.is_empty() {
            self.active = Some(0);
        }
    }

    #[allow(dead_code)]
    pub fn ribbon_ui(&mut self, ui: &mut egui::Ui, current_language: &Language) -> bool {
        let mut changed = false;
        ui.vertical(|ui| {
            ui.label(current_language.get_text(LanguageText::Shapes));
            egui::Frame::default()
                .fill(egui::Color32::WHITE)
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgb(218, 222, 228),
                ))
                .rounding(4.0)
                .inner_margin(egui::Margin::symmetric(4.0, 4.0))
                .show(ui, |ui| {
                    egui::Grid::new("shape_palette")
                        .num_columns(8)
                        .spacing([2.0, 2.0])
                        .show(ui, |ui| {
                            let mut clicked = None;
                            for (index, shape) in self.shapes.iter_mut().enumerate() {
                                if shape
                                    .shape_button(ui, current_language, self.active == Some(index))
                                    .clicked()
                                {
                                    clicked = Some(index);
                                }
                                if (index + 1) % 8 == 0 {
                                    ui.end_row();
                                }
                            }
                            if let Some(index) = clicked {
                                self.select(index);
                                changed = true;
                            }
                        });
                });
        });
        changed
    }
}

fn paint_shape_icon(
    painter: &egui::Painter,
    rect: egui::Rect,
    shape: ShapeKind,
    color: egui::Color32,
) {
    let stroke = egui::Stroke::new(1.25, color);
    let center = rect.center();
    let top = rect.top();
    let left = rect.left();
    let w = rect.width();
    let h = rect.height();
    let point = |x: f32, y: f32| egui::Pos2::new(left + x * w, top + y * h);

    let points = match shape {
        ShapeKind::Line => {
            painter.line_segment([point(0.1, 0.85), point(0.9, 0.15)], stroke);
            return;
        }
        ShapeKind::Curve => {
            draw_cubic_preview(
                painter,
                point(0.08, 0.78),
                point(0.28, 0.08),
                point(0.72, 0.94),
                point(0.92, 0.22),
                stroke,
            );
            return;
        }
        ShapeKind::Ellipse | ShapeKind::EllipseCallout => {
            painter.circle_stroke(center, w.min(h) * 0.43, stroke);
            if shape == ShapeKind::EllipseCallout {
                painter.line_segment([point(0.35, 0.85), point(0.15, 1.0)], stroke);
                painter.line_segment([point(0.15, 1.0), point(0.55, 0.85)], stroke);
            }
            return;
        }
        ShapeKind::Rectangle | ShapeKind::RoundedRectangle | ShapeKind::RectCallout => {
            painter.rect_stroke(rect.shrink(1.5), 1.5, stroke);
            if shape == ShapeKind::RectCallout {
                painter.line_segment([point(0.35, 0.92), point(0.16, 1.0)], stroke);
                painter.line_segment([point(0.16, 1.0), point(0.56, 0.92)], stroke);
            }
            return;
        }
        ShapeKind::Triangle => vec![point(0.5, 0.08), point(0.92, 0.9), point(0.08, 0.9)],
        ShapeKind::RightTriangle => vec![point(0.12, 0.1), point(0.88, 0.9), point(0.12, 0.9)],
        ShapeKind::Diamond => vec![
            point(0.5, 0.08),
            point(0.92, 0.5),
            point(0.5, 0.92),
            point(0.08, 0.5),
        ],
        ShapeKind::Pentagon => icon_regular(center, w, h, 5, -90.0),
        ShapeKind::Hexagon => icon_regular(center, w, h, 6, 30.0),
        ShapeKind::RightArrow => vec![
            point(0.08, 0.35),
            point(0.62, 0.35),
            point(0.62, 0.12),
            point(0.95, 0.5),
            point(0.62, 0.88),
            point(0.62, 0.65),
            point(0.08, 0.65),
        ],
        ShapeKind::LeftArrow => vec![
            point(0.92, 0.35),
            point(0.38, 0.35),
            point(0.38, 0.12),
            point(0.05, 0.5),
            point(0.38, 0.88),
            point(0.38, 0.65),
            point(0.92, 0.65),
        ],
        ShapeKind::UpArrow => vec![
            point(0.35, 0.92),
            point(0.35, 0.38),
            point(0.12, 0.38),
            point(0.5, 0.05),
            point(0.88, 0.38),
            point(0.65, 0.38),
            point(0.65, 0.92),
        ],
        ShapeKind::DownArrow => vec![
            point(0.35, 0.08),
            point(0.65, 0.08),
            point(0.65, 0.62),
            point(0.88, 0.62),
            point(0.5, 0.95),
            point(0.12, 0.62),
            point(0.35, 0.62),
        ],
        ShapeKind::FourPointStar => icon_star(center, w, h, 4),
        ShapeKind::FivePointStar => icon_star(center, w, h, 5),
        ShapeKind::SixPointStar => icon_star(center, w, h, 6),
        ShapeKind::Lightning => vec![
            point(0.62, 0.05),
            point(0.25, 0.5),
            point(0.48, 0.5),
            point(0.35, 0.95),
            point(0.82, 0.38),
            point(0.55, 0.38),
        ],
        ShapeKind::Polygon | ShapeKind::Extra => vec![
            point(0.25, 0.08),
            point(0.92, 0.32),
            point(0.76, 0.9),
            point(0.12, 0.74),
            point(0.08, 0.28),
        ],
    };

    for segment in points.windows(2) {
        painter.line_segment([segment[0], segment[1]], stroke);
    }
    if points.len() > 2 {
        painter.line_segment([points[points.len() - 1], points[0]], stroke);
    }
}

fn draw_cubic_preview(
    painter: &egui::Painter,
    p0: egui::Pos2,
    p1: egui::Pos2,
    p2: egui::Pos2,
    p3: egui::Pos2,
    stroke: egui::Stroke,
) {
    let mut previous = p0;
    for i in 1..=32 {
        let t = i as f32 / 32.0;
        let inv = 1.0 - t;
        let point = egui::Pos2::new(
            inv.powi(3) * p0.x
                + 3.0 * inv * inv * t * p1.x
                + 3.0 * inv * t * t * p2.x
                + t.powi(3) * p3.x,
            inv.powi(3) * p0.y
                + 3.0 * inv * inv * t * p1.y
                + 3.0 * inv * t * t * p2.y
                + t.powi(3) * p3.y,
        );
        painter.line_segment([previous, point], stroke);
        previous = point;
    }
}

fn icon_regular(
    center: egui::Pos2,
    width: f32,
    height: f32,
    sides: usize,
    rotation_deg: f32,
) -> Vec<egui::Pos2> {
    let rx = width * 0.42;
    let ry = height * 0.42;
    (0..sides)
        .map(|i| {
            let angle = (rotation_deg + i as f32 * 360.0 / sides as f32).to_radians();
            egui::Pos2::new(center.x + angle.cos() * rx, center.y + angle.sin() * ry)
        })
        .collect()
}

fn icon_star(center: egui::Pos2, width: f32, height: f32, points: usize) -> Vec<egui::Pos2> {
    let outer_rx = width * 0.44;
    let outer_ry = height * 0.44;
    let inner_rx = outer_rx * 0.42;
    let inner_ry = outer_ry * 0.42;

    (0..points * 2)
        .map(|i| {
            let angle = (-90.0 + i as f32 * 180.0 / points as f32).to_radians();
            let (rx, ry) = if i % 2 == 0 {
                (outer_rx, outer_ry)
            } else {
                (inner_rx, inner_ry)
            };
            egui::Pos2::new(center.x + angle.cos() * rx, center.y + angle.sin() * ry)
        })
        .collect()
}

impl Default for ShapeGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for ShapeGroup {
    fn get_tool_kind(&self) -> super::ToolKind {
        super::ToolKind::Shape
    }

    fn get_tool_label(&self, _current_language: &Language) -> LanguageText {
        crate::i18n::LanguageText::Shapes
    }
}
