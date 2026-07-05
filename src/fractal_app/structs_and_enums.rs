use egui::{Pos2, Vec2, emath::RectTransform};
use std::fmt;

// Fractal struct

#[derive(PartialEq, Eq, Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
pub enum LinesStyle {
    Free,
    Tree,
    Loop,
}
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum LineHandles {
    SingleHandle(usize),
    BothHandles, // or Both...
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct DesignLine {
    pub line: [Pos2; 2],
    pub reversed: bool,
}
#[derive(PartialEq, Clone, serde::Deserialize, serde::Serialize)]
pub struct Fractal {
    pub name: String,
    pub mirror: bool,
    pub rainbow: bool,
    pub design_lines: Vec<DesignLine>,
    pub replace_line: bool,
    pub lines_style: LinesStyle,
    pub zoom: f32,
    pub center: Pos2,
    pub start_line_width: f32,
    pub fixed_final_line_width: f32,
    pub depth: usize,
}

impl fmt::Debug for Fractal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // // Check if the user ran print via pretty-printed macro format "{:#?}"
        // if f.alternate() {
        write!(
            f,
            "name: {:?}.to_owned(),\n\
                mirror: {},\n\
                rainbow: {},\n\
                design_lines: vec![\n",
            self.name, self.mirror, self.rainbow
        )?;

        // Format each individual DesignLine vector entry manually
        for dl in &self.design_lines {
            writeln!(
                f,
                "    DesignLine {{ line: [pos2({:.1}, {:.1}), pos2({:.1}, {:.1})], reversed: {}, }},",
                dl.line[0].x, dl.line[0].y, dl.line[1].x, dl.line[1].y, dl.reversed
            )?;
        }

        write!(
            f,
            "],\n\
                replace_line: {},\n\
                lines_style: LinesStyle::{:?},\n\
                zoom: {},\n\
                center: pos2({:.1}, {:.1}),\n\
                start_line_width: {},\n\
                fixed_final_line_width: {},\n\
                depth: {},",
            self.replace_line,
            self.lines_style,
            self.zoom,
            self.center.x,
            self.center.y,
            self.start_line_width,
            self.fixed_final_line_width,
            self.depth
        )
        // } else {
    }
}

// design structs

#[derive(Clone, Copy)]
pub struct VectoredDesignLine {
    pub pos: Pos2,
    pub vec: Vec2,
}

impl VectoredDesignLine {
    pub fn from_design_line(
        DesignLine { line, reversed }: DesignLine,
        to_screen: RectTransform,
    ) -> Self {
        let (start, end) = if reversed {
            (to_screen * line[1], to_screen * line[0])
        } else {
            (to_screen * line[0], to_screen * line[1])
        };

        let vec = end - start;
        Self { pos: start, vec }
    }
}

// paint_fractal structs

#[derive(Clone, Copy)]
// OPTI!!!: add length and add length factor to LineTransform,
//   then the lengthfactor to calculate width does not need multiple length calcs
//   (pythagoras sqrt is expensive)
pub struct Node {
    pub pos: Pos2,
    pub vec: Vec2,
}

#[derive(Clone, Copy)]
pub struct LineTransform {
    pub base_rot: egui::emath::Rot2,
    pub rot: egui::emath::Rot2,
}

impl LineTransform {
    pub fn from_design_vector(
        base: &VectoredDesignLine,
        design_line: VectoredDesignLine,
        mirrored: bool,
    ) -> Self {
        let base_to_dcl: Vec2 = design_line.pos - base.pos;
        let mirror_sign: f32 = if mirrored { -1.0 } else { 1.0 };
        Self {
            base_rot: base_to_dcl.length() / base.vec.length()
                * egui::emath::Rot2::from_angle(
                    mirror_sign * (base_to_dcl.angle() - base.vec.angle()),
                ),
            rot: design_line.vec.length() / base.vec.length()
                * egui::emath::Rot2::from_angle(
                    mirror_sign * (design_line.vec.angle() - base.vec.angle()),
                ),
        }
    }
}
