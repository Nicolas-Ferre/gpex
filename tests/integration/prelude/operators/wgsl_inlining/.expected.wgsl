// INIT SHADER

struct Buffer {
    ident0: u32,
    ident1: u32,
    ident2: u32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident0 = u32(1);
    b.ident1 = u32(0);
    b.ident2 = u32(1);
}


// UPDATE SHADER

struct Buffer {
    ident0: u32,
    ident1: u32,
    ident2: u32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}
