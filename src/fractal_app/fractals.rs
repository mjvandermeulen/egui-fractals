use egui::pos2;

use super::{
    animation::animation_structs_and_enums::Animation,
    structs_and_enums::{Fractal, LinesStyle, ReversibleLine},
};

pub fn fractals() -> Vec<Fractal> {
    // in VSCode, rust-analyzer stumbles over returning a vec![] directly, so we assign it to a variable first
    let fractals = vec![
        Fractal {
            name: "Simple Twig".to_owned(),
            mirror: false,
            rainbow: false,
            design_lines: vec![
                ReversibleLine {
                    line: [pos2(0.0, 0.0), pos2(0.0, -1.0)],
                    reversed: false,
                },
                ReversibleLine {
                    line: [pos2(0.0, -1.0), pos2(0.5, -1.5)],
                    reversed: false,
                },
            ],
            replace_line: false,
            lines_style: LinesStyle::Free,
            zoom: 0.18,
            center: pos2(0.0, -2.5),
            initiator_width_length_ratio: 0.02,
            fixed_final_line_width: 1.0,
            depth: 9,
            animation: Animation { length: 1.5 },
        },
        Fractal {
            name: "Squares".to_owned(),
            mirror: false,
            rainbow: false,
            design_lines: vec![
                ReversibleLine {
                    line: [pos2(0.0, 0.0), pos2(0.0, -1.0)],
                    reversed: false,
                },
                ReversibleLine {
                    line: [pos2(0.1, -1.0), pos2(0.9, -1.0)],
                    reversed: false,
                },
                ReversibleLine {
                    line: [pos2(0.6, -1.0), pos2(0.6, -0.8)],
                    reversed: false,
                },
            ],
            replace_line: false,
            lines_style: LinesStyle::Free,
            zoom: 0.555_921_26,
            center: pos2(0.5, -0.7),
            initiator_width_length_ratio: 0.15,
            fixed_final_line_width: 1.0,
            depth: 14,
            animation: Animation { length: 1.5 },
        },
    ];
    fractals
}
