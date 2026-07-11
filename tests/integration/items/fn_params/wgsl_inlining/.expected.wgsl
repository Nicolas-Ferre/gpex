// INIT SHADER

struct Buffer {
    ident9: i32,
    ident12: f32,
    ident13: f32,
    ident14: f32,
    ident17: u32,
    ident18: u32,
    ident19: f32,
    ident21: f32,
    ident22: u32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident22 = ident23ident0(u32(4));
    b.ident21 = ident23ident1(f32(2.0));
    b.ident19 = ident20ident2();
    b.ident18 = ident20ident3();
    b.ident17 = ident20ident4();
    b.ident14 = ident15ident5(u32(5));
    b.ident13 = ident15ident6(u32(3));
    b.ident12 = ident15ident6(u32(2));
    b.ident9 = ident10ident7();
}

fn ident23ident0(ident24_const: u32) -> u32 {
    var ident24 = ident24_const;
    return ident24;
}

fn ident23ident1(ident24_const: f32) -> f32 {
    var ident24 = ident24_const;
    return ident24;
}

fn ident20ident2() -> f32 {
    return f32(3.0);
}

fn ident20ident3() -> u32 {
    return u32(2);
}

fn ident20ident4() -> u32 {
    return u32(1);
}

fn ident15ident5(ident16_const: u32) -> f32 {
    var ident16 = ident16_const;
    return f32(4.0);
}

fn ident15ident6(ident16_const: u32) -> f32 {
    var ident16 = ident16_const;
    return f32(1.0);
}

fn ident10ident7() -> i32 {
    return ident11ident8();
}

fn ident11ident8() -> i32 {
    return i32(1);
}


// UPDATE SHADER

struct Buffer {
    ident0: i32,
    ident1: f32,
    ident2: f32,
    ident3: f32,
    ident4: u32,
    ident5: u32,
    ident6: f32,
    ident7: f32,
    ident8: u32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}
