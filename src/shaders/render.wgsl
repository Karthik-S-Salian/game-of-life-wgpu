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
  grid_size:vec2f
}

@group(0) @binding(1) var<storage> cellState: array<u32>;

struct VertexOutput {
  @builtin(position) pos: vec4f,
  @location(0) cell: vec2f, // New line!
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;


@vertex fn display_vs(@builtin(vertex_index) vid: u32,@builtin(instance_index) instance: u32) ->VertexOutput {
  let i = f32(instance);
  let cell = vec2f(i%uniforms.grid_size.x, floor(i/uniforms.grid_size.y)); // Cell(1,1) in the image above
  let cellOffset = cell / uniforms.grid_size * 2.; // Compute the offset to cell
  
  let state = f32(cellState[instance]);
  let gridPos = (vertices[vid]*state*.8+1.)/uniforms.grid_size- 1.0 + cellOffset;


  var output: VertexOutput;
  output.pos = vec4f(gridPos, 0.0, 1.0);
  output.cell = cell; 
  return output;
}



@fragment
fn display_fs(input: VertexOutput) -> @location(0) vec4f {
  let c = input.cell /uniforms.grid_size;
  return vec4f(c, 1.-c.x, 1.);
}