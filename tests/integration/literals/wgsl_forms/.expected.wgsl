// INIT SHADER

struct Buffer {
    v11: u32,
    v12: u32,
    v13: f32,
    v14: f32,
    v15: f32,
    v16: f32,
    v17: i32,
    v18: i32,
    v19: i32,
    v20: u32,
    v21: u32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.v21 = u32(4294967295);
    b.v20 = u32(0);
    b.v19 = i32(2147483647);
    b.v18 = i32(0);
    b.v17 = i32(-2147483648);
    b.v16 = f32(3.4028235e38);
    b.v15 = f32(1e-23);
    b.v14 = f32(0.0);
    b.v13 = f32(-3.4028235e38);
    b.v12 = u32(1);
    b.v11 = u32(0);
}


// UPDATE SHADER

struct Buffer {
    v11: u32,
    v12: u32,
    v13: f32,
    v14: f32,
    v15: f32,
    v16: f32,
    v17: i32,
    v18: i32,
    v19: i32,
    v20: u32,
    v21: u32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}

