// INIT SHADER

struct Buffer {
    v5: vec2<u32>,
    v7: vec2<u32>,
    v9: vec2<u32>,
    v11: vec2<u32>,
    v13: vec2<u32>
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.v13 = vec2<u32>(0, 0);
    b.v11 = vec2<u32>(0, 4);
    b.v9 = vec2<u32>(0, 2);
    b.v7 = vec2<u32>(0, 1);
    b.v5 = vec2<u32>(0, 3);
}


// UPDATE SHADER

struct Buffer {
    v5: vec2<u32>,
    v7: vec2<u32>,
    v9: vec2<u32>,
    v11: vec2<u32>,
    v13: vec2<u32>
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}

