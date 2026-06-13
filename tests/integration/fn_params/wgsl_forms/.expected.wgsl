// INIT SHADER

struct Buffer {
    ident12: i32,
    ident15: f32,
    ident16: f32,
    ident17: f32,
    ident20: u32,
    ident21: u32,
    ident22: f32,
    ident25: u32,
    ident26: f32,
    ident30: u32,
    ident31: u32,
    ident32: f32,
    ident34: f32,
    ident35: u32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident35 = ident36ident0(u32(18));
    b.ident34 = ident36ident1(f32(16.0));
    b.ident32 = ident33ident2();
    b.ident31 = ident33ident3();
    b.ident30 = ident33ident4();
    b.ident26 = ident27ident5(f32(10.0), u32(11));
    b.ident25 = ident27ident6(u32(8), f32(9.0));
    b.ident22 = ident23ident7(f32(7.0));
    b.ident21 = ident23ident8(u32(6));
    b.ident20 = ident23ident8(u32(5));
    b.ident17 = ident18ident9(u32(4));
    b.ident16 = ident18ident10(u32(3));
    b.ident15 = ident18ident10(u32(2));
    b.ident12 = ident13ident11(i32(1));
}

fn ident36ident0(ident37_const: u32) -> u32 {
    var ident37 = ident37_const;
    return ident37;
}

fn ident36ident1(ident37_const: f32) -> f32 {
    var ident37 = ident37_const;
    return ident37;
}

fn ident33ident2() -> f32 {
    return f32(14.0);
}

fn ident33ident3() -> u32 {
    return u32(13);
}

fn ident33ident4() -> u32 {
    return u32(12);
}

fn ident27ident5(ident28_const: f32, ident29_const: u32) -> f32 {
    var ident28 = ident28_const;
    var ident29 = ident29_const;
    return ident28;
}

fn ident27ident6(ident28_const: u32, ident29_const: f32) -> u32 {
    var ident28 = ident28_const;
    var ident29 = ident29_const;
    return ident28;
}

fn ident23ident7(ident24_const: f32) -> f32 {
    var ident24 = ident24_const;
    return ident24;
}

fn ident23ident8(ident24_const: u32) -> u32 {
    var ident24 = ident24_const;
    return ident24;
}

fn ident18ident9(ident19_const: u32) -> f32 {
    var ident19 = ident19_const;
    return f32(3.0);
}

fn ident18ident10(ident19_const: u32) -> f32 {
    var ident19 = ident19_const;
    return f32(2.0);
}

fn ident13ident11(ident14_const: i32) -> i32 {
    var ident14 = ident14_const;
    return ident14;
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
    ident7: u32,
    ident8: f32,
    ident9: u32,
    ident10: u32,
    ident11: f32,
    ident12: f32,
    ident13: u32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}
