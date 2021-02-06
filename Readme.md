# Glow-imgui-Render

A dear imgui renderer using glow. Very early stage.

to create the renderer pass glow::Context and imgui::Context: 
```
let imgui_renderer = Renderer::new(&gl, &mut imgui);
```

to draw, just pass the glow::Context and the imgui draw data: 

```
let draw_data = ui.render();
imgui_renderer.render(&gl, &draw_data);
```

For more, please look at the basic example using glow and glutin:
```
cargo run --example basic
```


### Todo´s: 

1. Add texture suppot
2. Save and restore previous opengl context on draw function...
3. Add webgl support.




