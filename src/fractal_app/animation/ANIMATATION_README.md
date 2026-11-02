# Claude

```mermaid
flowchart TD
    A["fractal.design_lines<br/>(long-lived, borrowed)"]
    B["reversible_lines_to_global_line_vectors(&amp;lines, to_screen)<br/>borrow in, owned out"]
    C["global_design_vectors<br/>Vec&lt;Line&gt;"]
    D{"show_design_only?"}
    E["paint_design(&amp;global_design_vectors)<br/>borrow"]
    F{"animation_state"}
    G["Stopped:<br/>paint_vectors = global_design_vectors<br/>(move)"]
    H["Otherwise:<br/>paint_vectors = animation_frame_vectors(global_design_vectors, progress, cycle)<br/>(consume in, owned out)"]
    I["paint_vectors<br/>Vec&lt;Line&gt;"]
    J["paint_fractal(&amp;painter, paint_vectors)"]

    A --> B --> C --> D
    D -- yes --> E
    D -- no --> F
    F -- Stopped --> G
    F -- animating --> H
    G --> I
    H --> I
    I --> J
```

The two branches merge into `paint_vectors`, which is the point of the name: it's defined by its role (the input to `paint_fractal`), not by how it was produced. I put the ownership notes (borrow, move, consume) on the nodes so the diagram doubles as a map of where each value is used up. If you'd rather have a cleaner diagram, delete the parenthetical lines.
