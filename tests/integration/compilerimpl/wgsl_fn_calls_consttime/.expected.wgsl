// INIT SHADER

struct Buffer {
    ident0: u32,
    ident1: u32,
    ident2: u32,
    ident3: u32,
    ident4: u32,
    ident5: i32,
    ident6: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident6 = i32(-2147483648);
    b.ident5 = i32(3);
    b.ident4 = u32(8);
    b.ident3 = u32(4);
    b.ident2 = u32(4);
    b.ident1 = u32(4);
    b.ident0 = u32(4);
}


// UPDATE SHADER

struct Buffer {
    ident0: u32,
    ident1: u32,
    ident2: u32,
    ident3: u32,
    ident4: u32,
    ident5: i32,
    ident6: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}
