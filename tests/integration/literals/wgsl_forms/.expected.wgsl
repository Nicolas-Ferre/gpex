// INIT SHADER

struct Buffer {
    ident0: u32,
    ident1: u32,
    ident2: f32,
    ident3: f32,
    ident4: f32,
    ident5: f32,
    ident6: i32,
    ident7: i32,
    ident8: i32,
    ident9: u32,
    ident10: u32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident10 = u32(4294967295);
    b.ident9 = u32(0);
    b.ident8 = i32(2147483647);
    b.ident7 = i32(0);
    b.ident6 = i32(-2147483648);
    b.ident5 = f32(3.4028235e38);
    b.ident4 = f32(1e-23);
    b.ident3 = f32(0.0);
    b.ident2 = f32(-3.4028235e38);
    b.ident1 = u32(1);
    b.ident0 = u32(0);
}


// UPDATE SHADER

struct Buffer {
    ident0: u32,
    ident1: u32,
    ident2: f32,
    ident3: f32,
    ident4: f32,
    ident5: f32,
    ident6: i32,
    ident7: i32,
    ident8: i32,
    ident9: u32,
    ident10: u32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}
