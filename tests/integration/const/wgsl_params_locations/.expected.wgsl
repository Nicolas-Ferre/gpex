// INIT SHADER

struct Buffer {
    v7: i32,
    v16: i32,
    v26: i32,
    v39: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.v39 = i32(4);
    b.v26 = i32(3);
    b.v16 = i32(2);
    b.v7 = i32(1);
}


// UPDATE SHADER

struct Buffer {
    v7: i32,
    v16: i32,
    v26: i32,
    v39: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}

