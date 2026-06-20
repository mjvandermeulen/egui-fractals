/// cap depth logarithmically `branches`^`depth` < `max_painted_line_count`
///   when replacing
/// it gets a little more complicated when not replacing
pub fn max_depth_with_branches(
    max_painted_line_count: usize,
    design_branches_count: usize,
    mirror: bool,
    replace_line: bool,
) -> usize {
    let branch_count = if mirror {
        design_branches_count * 2
    } else {
        design_branches_count
    };
    // If b is 1, each term is 1, so maximum power d is maximum - 1
    if branch_count < 2 {
        return max_painted_line_count - 1;
    }

    // `branches` to the power of `max_depth` < `max_painted_line_count`
    if replace_line {
        return (max_painted_line_count as f32)
            .log(branch_count as f32)
            .floor() as usize;
    }

    // copied from gemini
    let argument: usize = max_painted_line_count * (branch_count - 1) + 1;
    if argument == 0 {
        return 0;
    }
    let max_depth_float = ((argument as f32).ln() / (branch_count as f32).ln()) - 1.0;

    max_depth_float.floor() as usize
}
