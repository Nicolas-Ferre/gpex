// INIT SHADER

struct Buffer {
    v8: i32,
    v12: vec2<u32>,
    v16: i32,
    v25: i32,
    v31: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.v31 = i32(4);
    b.v25 = i32(3);
    b.v16 = i32(2);
    b.v12 = vec2<u32>(0, 1);
    b.v8 = i32(1);
}


// UPDATE SHADER

struct Buffer {
    v8: i32,
    v12: vec2<u32>,
    v16: i32,
    v25: i32,
    v31: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}

