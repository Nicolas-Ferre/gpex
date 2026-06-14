// INIT SHADER

struct Buffer {
    ident14: i32,
    ident17: f32,
    ident18: f32,
    ident19: f32,
    ident22: u32,
    ident23: u32,
    ident24: f32,
    ident27: u32,
    ident28: f32,
    ident32: u32,
    ident33: u32,
    ident34: f32,
    ident36: f32,
    ident37: u32,
    ident40: u32,
    ident41: f32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident41 = ident42ident0(f32(21.0), f32(22.0));
    b.ident40 = ident42ident1(u32(19), u32(20));
    b.ident37 = ident38ident2(u32(18));
    b.ident36 = ident38ident3(f32(16.0));
    b.ident34 = ident35ident4();
    b.ident33 = ident35ident5();
    b.ident32 = ident35ident6();
    b.ident28 = ident29ident7(f32(10.0), u32(11));
    b.ident27 = ident29ident8(u32(8), f32(9.0));
    b.ident24 = ident25ident9(f32(7.0));
    b.ident23 = ident25ident10(u32(6));
    b.ident22 = ident25ident10(u32(5));
    b.ident19 = ident20ident11(u32(4));
    b.ident18 = ident20ident12(u32(3));
    b.ident17 = ident20ident12(u32(2));
    b.ident14 = ident15ident13(i32(1));
}

fn ident42ident0(ident43_const: f32, ident44_const: f32) -> f32 {
    var ident43 = ident43_const;
    var ident44 = ident44_const;
    return ident44;
}

fn ident42ident1(ident43_const: u32, ident44_const: u32) -> u32 {
    var ident43 = ident43_const;
    var ident44 = ident44_const;
    return ident44;
}

fn ident38ident2(ident39_const: u32) -> u32 {
    var ident39 = ident39_const;
    return ident39;
}

fn ident38ident3(ident39_const: f32) -> f32 {
    var ident39 = ident39_const;
    return ident39;
}

fn ident35ident4() -> f32 {
    return f32(14.0);
}

fn ident35ident5() -> u32 {
    return u32(13);
}

fn ident35ident6() -> u32 {
    return u32(12);
}

fn ident29ident7(ident30_const: f32, ident31_const: u32) -> f32 {
    var ident30 = ident30_const;
    var ident31 = ident31_const;
    return ident30;
}

fn ident29ident8(ident30_const: u32, ident31_const: f32) -> u32 {
    var ident30 = ident30_const;
    var ident31 = ident31_const;
    return ident30;
}

fn ident25ident9(ident26_const: f32) -> f32 {
    var ident26 = ident26_const;
    return ident26;
}

fn ident25ident10(ident26_const: u32) -> u32 {
    var ident26 = ident26_const;
    return ident26;
}

fn ident20ident11(ident21_const: u32) -> f32 {
    var ident21 = ident21_const;
    return f32(3.0);
}

fn ident20ident12(ident21_const: u32) -> f32 {
    var ident21 = ident21_const;
    return f32(2.0);
}

fn ident15ident13(ident16_const: i32) -> i32 {
    var ident16 = ident16_const;
    return ident16;
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
    ident13: u32,
    ident14: u32,
    ident15: f32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}
