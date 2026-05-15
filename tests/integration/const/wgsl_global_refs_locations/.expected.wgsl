// INIT SHADER

struct Buffer {
    v17: i32,
    v21: u32,
    v31: f32,
    v41: i32,
    v48: i32,
    v60: i32,
    v65: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.v65 = i32(7);
    b.v60 = i32(6);
    b.v48 = i32(5);
    b.v41 = i32(4);
    b.v31 = f32(3.0);
    b.v21 = u32(2);
    b.v17 = i32(1);
}


// UPDATE SHADER

struct Buffer {
    v17: i32,
    v21: u32,
    v31: f32,
    v41: i32,
    v48: i32,
    v60: i32,
    v65: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}

