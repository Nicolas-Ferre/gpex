// INIT SHADER

struct Buffer {
    ident0: i32,
    ident1: i32,
    ident2: i32,
    ident3: u32,
    ident4: f32,
    ident5: u32,
    ident6: f32,
    ident7: u32,
    ident8: u32,
    ident9: f32,
    ident10: i32,
    ident11: u32,
    ident12: u32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident12 = u32(1);
    b.ident11 = u32(14);
    b.ident10 = i32(13);
    b.ident9 = f32(12.0);
    b.ident8 = u32(10);
    b.ident7 = u32(4);
    b.ident6 = f32(7.0);
    b.ident5 = u32(6);
    b.ident4 = f32(5.0);
    b.ident3 = u32(4);
    b.ident2 = i32(3);
    b.ident1 = i32(2);
    b.ident0 = i32(1);
}


// UPDATE SHADER

struct Buffer {
    ident0: i32,
    ident1: i32,
    ident2: i32,
    ident3: u32,
    ident4: f32,
    ident5: u32,
    ident6: f32,
    ident7: u32,
    ident8: u32,
    ident9: f32,
    ident10: i32,
    ident11: u32,
    ident12: u32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}
