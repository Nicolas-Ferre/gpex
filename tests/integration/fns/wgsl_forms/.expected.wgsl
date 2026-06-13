// INIT SHADER

struct Buffer {
    ident10: i32,
    ident11: u32,
    ident14: u32,
    ident15: f32,
    ident20: u32,
    ident21: f32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident21 = ident22ident0();
    b.ident20 = ident22ident1();
    b.ident15 = ident16ident2(f32(4.0));
    b.ident14 = ident16ident3(u32(3));
    b.ident11 = ident12ident4(u32(2));
    b.ident10 = ident12ident5(i32(1));
}

fn ident22ident0() -> f32 {
    return ident23ident6(f32(2.0));
}

fn ident22ident1() -> u32 {
    return ident23ident7(u32(1));
}

fn ident16ident2(ident17_const: f32) -> f32 {
    var ident17 = ident17_const;
    return ident18ident8(ident17);
}

fn ident16ident3(ident17_const: u32) -> u32 {
    var ident17 = ident17_const;
    return ident18ident9(ident17);
}

fn ident12ident4(ident13_const: u32) -> u32 {
    var ident13 = ident13_const;
    return ident13;
}

fn ident12ident5(ident13_const: i32) -> i32 {
    var ident13 = ident13_const;
    return ident13;
}

fn ident23ident6(ident24_const: f32) -> f32 {
    var ident24 = ident24_const;
    return ident24;
}

fn ident23ident7(ident24_const: u32) -> u32 {
    var ident24 = ident24_const;
    return ident24;
}

fn ident18ident8(ident19_const: f32) -> f32 {
    var ident19 = ident19_const;
    return ident19;
}

fn ident18ident9(ident19_const: u32) -> u32 {
    var ident19 = ident19_const;
    return ident19;
}


// UPDATE SHADER

struct Buffer {
    ident0: i32,
    ident1: u32,
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
