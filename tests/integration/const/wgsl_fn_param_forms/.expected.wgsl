// INIT SHADER

struct Buffer {
    ident0: i32,
    ident1: i32,
    ident2: u32,
    ident3: f32,
    ident4: u32,
    ident5: f32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident5 = f32(6.0);
    b.ident4 = u32(5);
    b.ident3 = f32(4.0);
    b.ident2 = u32(3);
    b.ident1 = i32(2);
    b.ident0 = i32(1);
}


// UPDATE SHADER

struct Buffer {
    ident0: i32,
    ident1: i32,
    ident2: u32,
    ident3: f32,
    ident4: u32,
    ident5: f32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}
