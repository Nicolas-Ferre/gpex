// INIT SHADER

struct Buffer {
    ident0: i32,
    ident1: i32,
    ident2: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident0 = i32(1);
    b.ident1 = i32(2) + b.ident0;
    b.ident2 = i32(2147483647) + b.ident0;
}


// UPDATE SHADER

struct Buffer {
    ident0: i32,
    ident1: i32,
    ident2: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}
