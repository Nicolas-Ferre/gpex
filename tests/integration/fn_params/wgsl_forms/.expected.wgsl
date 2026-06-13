// INIT SHADER

struct Buffer {
    ident10: i32,
    ident13: f32,
    ident14: f32,
    ident15: f32,
    ident18: i32,
    ident19: i32,
    ident20: i32,
    ident23: i32,
    ident24: i32,
    ident28: i32,
    ident29: i32,
    ident30: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident30 = ident31ident0();
    b.ident29 = ident31ident1();
    b.ident28 = ident31ident2();
    b.ident24 = ident25ident3(f32(10.0), u32(11));
    b.ident23 = ident25ident4(u32(8), f32(9.0));
    b.ident20 = ident21ident5(f32(7.0));
    b.ident19 = ident21ident6(u32(6));
    b.ident18 = ident21ident6(u32(5));
    b.ident15 = ident16ident7(u32(4));
    b.ident14 = ident16ident8(u32(3));
    b.ident13 = ident16ident8(u32(2));
    b.ident10 = ident11ident9(i32(1));
}

fn ident31ident0() -> i32 {
    return i32(7);
}

fn ident31ident1() -> i32 {
    return i32(7);
}

fn ident31ident2() -> i32 {
    return i32(7);
}

fn ident25ident3(ident26_const: f32, ident27_const: u32) -> i32 {
    var ident26 = ident26_const;
    var ident27 = ident27_const;
    return i32(6);
}

fn ident25ident4(ident26_const: u32, ident27_const: f32) -> i32 {
    var ident26 = ident26_const;
    var ident27 = ident27_const;
    return i32(6);
}

fn ident21ident5(ident22_const: f32) -> i32 {
    var ident22 = ident22_const;
    return i32(5);
}

fn ident21ident6(ident22_const: u32) -> i32 {
    var ident22 = ident22_const;
    return i32(5);
}

fn ident16ident7(ident17_const: u32) -> f32 {
    var ident17 = ident17_const;
    return f32(3.0);
}

fn ident16ident8(ident17_const: u32) -> f32 {
    var ident17 = ident17_const;
    return f32(2.0);
}

fn ident11ident9(ident12_const: i32) -> i32 {
    var ident12 = ident12_const;
    return ident12;
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
    ident7: i32,
    ident8: i32,
    ident9: i32,
    ident10: i32,
    ident11: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}
