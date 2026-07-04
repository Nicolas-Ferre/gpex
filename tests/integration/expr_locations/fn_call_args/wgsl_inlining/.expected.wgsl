// INIT SHADER

struct Buffer {
    ident0: i32,
    ident1: i32,
    ident2: u32,
    ident3: i32,
    ident4: i32,
    ident5: u32,
    ident6: i32,
    ident7: u32,
    ident8: u32,
    ident9: f32,
    ident10: u32,
    ident11: u32,
    ident12: f32,
    ident13: u32,
    ident14: f32,
    ident15: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident15 = i32(1);
    b.ident14 = f32(2.0);
    b.ident13 = u32(1);
    b.ident12 = f32(4.0);
    b.ident11 = u32(2);
    b.ident10 = u32(4);
    b.ident9 = f32(2.0);
    b.ident8 = u32(1);
    b.ident7 = u32(2);
    b.ident6 = i32(1);
    b.ident5 = u32(1);
    b.ident4 = i32(1);
    b.ident3 = i32(1);
    b.ident2 = u32(2);
    b.ident1 = i32(1);
    b.ident0 = i32(1);
}


// UPDATE SHADER

struct Buffer {
    ident0: i32,
    ident1: i32,
    ident2: u32,
    ident3: i32,
    ident4: i32,
    ident5: u32,
    ident6: i32,
    ident7: u32,
    ident8: u32,
    ident9: f32,
    ident10: u32,
    ident11: u32,
    ident12: f32,
    ident13: u32,
    ident14: f32,
    ident15: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}
