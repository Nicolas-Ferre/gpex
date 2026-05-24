// INIT SHADER

struct Buffer {
    v13: i32,
    v22: i32,
    v32: i32,
    v45: i32,
    v60: i32,
    v75: i32,
    v92: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.v92 = i32(6);
    b.v75 = i32(5);
    b.v60 = i32(5);
    b.v45 = i32(4);
    b.v32 = i32(3);
    b.v22 = i32(2);
    b.v13 = i32(1);
}


// UPDATE SHADER

struct Buffer {
    v13: i32,
    v22: i32,
    v32: i32,
    v45: i32,
    v60: i32,
    v75: i32,
    v92: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}

