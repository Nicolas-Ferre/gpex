// INIT SHADER

struct Buffer {
    ident7: i32,
    ident10: f32,
    ident11: f32,
    ident12: f32,
    ident15: i32,
    ident16: i32,
    ident19: i32,
    ident20: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident20 = ident21ident0();
    b.ident19 = ident21ident1();
    b.ident16 = ident17ident2(f32(6.0));
    b.ident15 = ident17ident3(u32(5));
    b.ident12 = ident13ident4(u32(4));
    b.ident11 = ident13ident5(u32(3));
    b.ident10 = ident13ident5(u32(2));
    b.ident7 = ident8ident6(i32(1));
}

fn ident21ident0() -> i32 {
    return i32(8);
}

fn ident21ident1() -> i32 {
    return i32(8);
}

fn ident17ident2(ident18_const: f32) -> i32 {
    var ident18 = ident18_const;
    return i32(5);
}

fn ident17ident3(ident18_const: u32) -> i32 {
    var ident18 = ident18_const;
    return i32(5);
}

fn ident13ident4(ident14_const: u32) -> f32 {
    var ident14 = ident14_const;
    return f32(3.0);
}

fn ident13ident5(ident14_const: u32) -> f32 {
    var ident14 = ident14_const;
    return f32(2.0);
}

fn ident8ident6(ident9_const: i32) -> i32 {
    var ident9 = ident9_const;
    return ident9;
}


// UPDATE SHADER

struct Buffer {
    ident0: i32,
    ident1: f32,
    ident2: f32,
    ident3: f32,
    ident4: i32,
    ident5: i32,
    ident6: i32,
    ident7: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}
