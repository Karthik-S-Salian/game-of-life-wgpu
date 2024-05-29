@group(0) @binding(0) var<uniform> grid: vec2f;
@group(0) @binding(1) var<storage> cellStateIn: array<u32>;
@group(0) @binding(2) var<storage, read_write> cellStateOut: array<u32>;

fn cellActive(x: u32, y: u32) -> u32 {
return cellStateIn[cellIndex(vec2(x, y))];
}

fn cellIndex(cell: vec2u) -> u32 {
return (cell.y % u32(grid.y)) * u32(grid.y) + (cell.x % u32(grid.x));  //to handle edge overflow cases
}

@compute
@workgroup_size(8,8) // New line
fn computeMain(@builtin(global_invocation_id) cell: vec3u) {
let activeNeighbors = cellActive(cell.x + 1u, cell.y+ 1u) +
  cellActive(cell.x+1u, cell.y) +
  cellActive(cell.x+1u, cell.y- 1u) +
  cellActive(cell.x, cell.y- 1u) +
  cellActive(cell.x- 1u, cell.y- 1u) +
  cellActive(cell.x- 1u, cell.y) +
  cellActive(cell.x- 1u, cell.y+ 1u) +
  cellActive(cell.x, cell.y+1u);

let i = cellIndex(cell.xy);

// Conway's game of life rules:
switch activeNeighbors {
  case 2u: { // Active cells with 2 neighbors stay active.
    cellStateOut[i] = cellStateIn[i];
  }
  case 3u: { // Cells with 3 neighbors become or stay active.
    cellStateOut[i] = 1u;
  }
  default: { // Cells with < 2 or > 3 neighbors become inactive.
    cellStateOut[i] = 0u;
  }
}
    // if cellStateIn[i]==1u{
    //     cellStateOut[i]=0u;
    // }else{
    //     cellStateOut[i]=1u;
    // }
}