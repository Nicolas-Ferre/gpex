// INIT SHADER

struct Buffer {
    ident5: i32,
    ident8: f32,
    ident9: f32,
    ident10: f32,
    ident13: i32,
    ident14: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident14 = ident15ident0(f32(6.0));
    b.ident13 = ident15ident1(u32(5));
    b.ident10 = ident11ident2(u32(4));
    b.ident9 = ident11ident3(u32(3));
    b.ident8 = ident11ident3(u32(2));
    b.ident5 = ident6ident4(i32(1));
}

fn ident15ident0(ident16_const: f32) -> i32 {
    var ident16 = ident16_const;
    return i32(5);
}

fn ident15ident1(ident16_const: u32) -> i32 {
    var ident16 = ident16_const;
    return i32(5);
}

fn ident11ident2(ident12_const: u32) -> f32 {
    var ident12 = ident12_const;
    return f32(3.0);
}

fn ident11ident3(ident12_const: u32) -> f32 {
    var ident12 = ident12_const;
    return f32(2.0);
}

fn ident6ident4(ident7_const: i32) -> i32 {
    var ident7 = ident7_const;
    return ident7;
}


// UPDATE SHADER

struct Buffer {
    ident0: i32,
    ident1: f32,
    ident2: f32,
    ident3: f32,
    ident4: i32,
    ident5: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}
