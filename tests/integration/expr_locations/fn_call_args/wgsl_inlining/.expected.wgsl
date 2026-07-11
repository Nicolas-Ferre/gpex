// INIT SHADER

struct Buffer {
    ident0: i32,
    ident1: i32,
    ident2: i32,
    ident3: i32,
    ident4: u32,
    ident5: i32,
    ident6: i32,
    ident7: u32,
    ident8: i32,
    ident9: u32,
    ident10: u32,
    ident11: f32,
    ident12: u32,
    ident13: u32,
    ident14: f32,
    ident15: u32,
    ident16: f32,
    ident17: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident17 = i32(1);
    b.ident16 = f32(2.0);
    b.ident15 = u32(1);
    b.ident14 = f32(4.0);
    b.ident13 = u32(2);
    b.ident12 = u32(4);
    b.ident11 = f32(2.0);
    b.ident10 = u32(1);
    b.ident9 = u32(2);
    b.ident8 = i32(1);
    b.ident7 = u32(1);
    b.ident6 = i32(1);
    b.ident5 = i32(1);
    b.ident4 = u32(2);
    b.ident3 = i32(1);
    b.ident2 = i32(1);
    b.ident1 = i32(1);
    b.ident0 = i32(1);
}


// UPDATE SHADER

struct Buffer {
    ident0: i32,
    ident1: i32,
    ident2: i32,
    ident3: i32,
    ident4: u32,
    ident5: i32,
    ident6: i32,
    ident7: u32,
    ident8: i32,
    ident9: u32,
    ident10: u32,
    ident11: f32,
    ident12: u32,
    ident13: u32,
    ident14: f32,
    ident15: u32,
    ident16: f32,
    ident17: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}
