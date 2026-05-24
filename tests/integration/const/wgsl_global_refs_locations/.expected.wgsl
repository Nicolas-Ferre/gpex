// INIT SHADER

struct Buffer {
    v23: i32,
    v27: u32,
    v37: f32,
    v47: i32,
    v54: i32,
    v66: i32,
    v71: i32,
    v76: i32,
    v86: i32,
    v98: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.v98 = i32(10);
    b.v86 = i32(9);
    b.v76 = i32(8);
    b.v71 = i32(7);
    b.v66 = i32(6);
    b.v54 = i32(5);
    b.v47 = i32(4);
    b.v37 = f32(3.0);
    b.v27 = u32(2);
    b.v23 = i32(1);
}


// UPDATE SHADER

struct Buffer {
    v23: i32,
    v27: u32,
    v37: f32,
    v47: i32,
    v54: i32,
    v66: i32,
    v71: i32,
    v76: i32,
    v86: i32,
    v98: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}

