alias TriangleVertices = array<vec2f, 6>;
var<private> vertices: TriangleVertices = TriangleVertices(
  vec2f(-1.0,  1.0),
  vec2f(-1.0, -1.0),
  vec2f( 1.0,  1.0),
  vec2f( 1.0,  1.0),
  vec2f(-1.0, -1.0),
  vec2f( 1.0, -1.0),
);

struct Uniforms {
  grid_size:f32
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;


@vertex fn display_vs(@builtin(vertex_index) vid: u32,@builtin(instance_index) instance: u32) -> @builtin(position) vec4f {
  let i = f32(instance);
  let cell = vec2f(i%uniforms.grid_size, floor(i/uniforms.grid_size)); // Cell(1,1) in the image above
  let cellOffset = cell / uniforms.grid_size * 2.; // Compute the offset to cell
  let gridPos = (vertices[vid]*.8+1.)/uniforms.grid_size- 1.0 + cellOffset;
  return vec4f(gridPos, 0.0, 1.0);
}


@fragment fn display_fs(@builtin(position) pos: vec4f) -> @location(0) vec4f {
  return vec4f(1.,0., 0.0, 1.0);
}