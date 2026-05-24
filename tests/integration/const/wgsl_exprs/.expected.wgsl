// INIT SHADER

struct Buffer {
    v14: i32,
    v18: vec2<u32>,
    v22: i32,
    v31: i32,
    v37: i32,
    v46: i32,
    v54: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.v54 = i32(6);
    b.v46 = i32(5);
    b.v37 = i32(4);
    b.v31 = i32(3);
    b.v22 = i32(2);
    b.v18 = vec2<u32>(0, 1);
    b.v14 = i32(1);
}


// UPDATE SHADER

struct Buffer {
    v14: i32,
    v18: vec2<u32>,
    v22: i32,
    v31: i32,
    v37: i32,
    v46: i32,
    v54: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}

