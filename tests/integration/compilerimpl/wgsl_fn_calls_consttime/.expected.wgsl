// INIT SHADER

struct Buffer {
    v13: i32,
    v17: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.v17 = i32(-2147483648);
    b.v13 = i32(3);
}


// UPDATE SHADER

struct Buffer {
    v13: i32,
    v17: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}

