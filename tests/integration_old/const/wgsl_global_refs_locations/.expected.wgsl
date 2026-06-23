// INIT SHADER

struct Buffer {
    ident0: i32,
    ident1: u32,
    ident2: i32,
    ident3: f32,
    ident4: i32,
    ident5: i32,
    ident6: i32,
    ident7: i32,
    ident8: i32,
    ident9: i32,
    ident10: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident10 = i32(11);
    b.ident9 = i32(10);
    b.ident8 = i32(9);
    b.ident7 = i32(8);
    b.ident6 = i32(7);
    b.ident5 = i32(6);
    b.ident4 = i32(5);
    b.ident3 = f32(4.0);
    b.ident2 = i32(3);
    b.ident1 = u32(2);
    b.ident0 = i32(1);
}


// UPDATE SHADER

struct Buffer {
    ident0: i32,
    ident1: u32,
    ident2: i32,
    ident3: f32,
    ident4: i32,
    ident5: i32,
    ident6: i32,
    ident7: i32,
    ident8: i32,
    ident9: i32,
    ident10: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}
