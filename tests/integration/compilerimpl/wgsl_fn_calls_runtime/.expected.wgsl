// INIT SHADER

struct Buffer {
    v11: i32,
    v12: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.v11 = i32(1);
    b.v12 = b.v11 + i32(2);
}


// UPDATE SHADER

struct Buffer {
    v11: i32,
    v12: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}

