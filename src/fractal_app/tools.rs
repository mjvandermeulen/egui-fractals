/// cap depth logarithmically `branches`^`depth` < `max_painted_line_count`
pub fn max_depth_with_branches(
    max_painted_line_count: usize,
    num_design_branches: usize,
    mirror: bool,
) -> usize {
    // copied from gemini
    let construction_branches = if mirror {
        num_design_branches * 2
    } else {
        num_design_branches
    };
    // If b is 1, each term is 1, so maximum power d is maximum - 1
    if construction_branches < 2 {
        return max_painted_line_count - 1;
    }

    let argument: usize = max_painted_line_count * (construction_branches - 1) + 1;

    if argument == 0 {
        return 0;
    }

    let max_depth_float = ((argument as f32).ln() / (construction_branches as f32).ln()) - 1.0;

    max_depth_float.floor() as usize
}
