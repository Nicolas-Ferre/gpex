// INIT SHADER

struct Buffer {
    ident0: u32,
    ident1: u32,
    ident2: u32,
    ident3: u32,
    ident4: u32,
    ident5: u32,
    ident6: vec2<u32>,
    ident7: vec2<u32>,
    ident8: vec2<u32>,
    ident9: vec2<u32>,
    ident10: i32,
    ident11: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident11 = i32(-2147483648);
    b.ident10 = i32(3);
    b.ident9 = vec2<u32>(0, 2);
    b.ident7 = vec2<u32>(0, 0);
    b.ident6 = vec2<u32>(0, 1);
    b.ident5 = u32(8);
    b.ident4 = u32(4);
    b.ident3 = u32(4);
    b.ident2 = u32(4);
    b.ident1 = u32(4);
    b.ident0 = u32(2);
    b.ident8 = vec2<u32>(0, 2);
}


// UPDATE SHADER

struct Buffer {
    ident0: u32,
    ident1: u32,
    ident2: u32,
    ident3: u32,
    ident4: u32,
    ident5: u32,
    ident6: vec2<u32>,
    ident7: vec2<u32>,
    ident8: vec2<u32>,
    ident9: vec2<u32>,
    ident10: i32,
    ident11: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}
