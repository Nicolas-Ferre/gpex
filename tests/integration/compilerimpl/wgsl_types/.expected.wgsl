// INIT SHADER

struct Buffer {
    v11: vec2<u32>,
    v13: vec2<u32>,
    v15: vec2<u32>,
    v17: vec2<u32>,
    v19: vec2<u32>
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.v19 = vec2<u32>(0, 0);
    b.v17 = vec2<u32>(0, 4);
    b.v15 = vec2<u32>(0, 2);
    b.v13 = vec2<u32>(0, 1);
    b.v11 = vec2<u32>(0, 3);
}


// UPDATE SHADER

struct Buffer {
    v11: vec2<u32>,
    v13: vec2<u32>,
    v15: vec2<u32>,
    v17: vec2<u32>,
    v19: vec2<u32>
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}

