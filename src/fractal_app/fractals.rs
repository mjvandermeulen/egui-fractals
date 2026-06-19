use egui::pos2;

use crate::fractal_app::structs::{DesignLine, Fractal, LinesStyle};

pub fn fractals() -> Vec<Fractal> {
    vec![
        Fractal {
            mirror: false,
            rainbow: false,
            design_line_count: 1,
            design_lines: vec![
                DesignLine {
                    line: [pos2(0.0, 0.0), pos2(0.0, -1.0)],
                    reversed: false,
                },
                DesignLine {
                    line: [pos2(0.0, -1.0), pos2(0.5, -1.5)],
                    reversed: false,
                },
                DesignLine {
                    line: [pos2(0.0, -1.0), pos2(0.0, -1.70)],
                    reversed: false,
                },
                DesignLine {
                    line: [pos2(0.0, -1.0), pos2(-0.5, -1.5)],
                    reversed: false,
                },
                DesignLine {
                    line: [pos2(0.0, -1.0), pos2(-0.5, -0.5)],
                    reversed: false,
                },
            ],
            replace_line: false,
            lines_style: LinesStyle::Free,
            zoom: 0.18,
            center: pos2(0.0, -2.5),
            start_line_width: 2.5, // TODO strangely global screen coords width... prob OK. Has to be visible
            fixed_final_line_width: 1.0,
            depth: 9,
        },
        Fractal {
            mirror: false,
            rainbow: false,
            design_line_count: 2,
            design_lines: vec![
                DesignLine {
                    line: [pos2(0.0, 0.0), pos2(0.0, -1.0)],
                    reversed: false,
                },
                DesignLine {
                    line: [pos2(0.1, -1.0), pos2(0.9, -1.0)],
                    reversed: false,
                },
                DesignLine {
                    line: [pos2(0.6, -1.0), pos2(0.6, -0.8)],
                    reversed: false,
                },
                DesignLine {
                    line: [pos2(0.0, -1.0), pos2(0.5, -1.5)],
                    reversed: false,
                },
                DesignLine {
                    line: [pos2(0.0, -1.0), pos2(-0.5, -0.5)],
                    reversed: false,
                },
            ],
            replace_line: false,
            lines_style: LinesStyle::Free,
            zoom: 0.55592126,
            center: pos2(0.5, -0.7),
            start_line_width: 6.0,
            fixed_final_line_width: 1.0,
            depth: 17,
        },
    ]
}
