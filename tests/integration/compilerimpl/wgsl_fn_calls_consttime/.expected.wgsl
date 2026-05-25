// INIT SHADER

struct Buffer {
    ident0: i32,
    ident1: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident1 = i32(-2147483648);
    b.ident0 = i32(3);
}


// UPDATE SHADER

struct Buffer {
    ident0: i32,
    ident1: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}

