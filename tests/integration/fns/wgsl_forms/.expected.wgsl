// INIT SHADER

struct Buffer {
    ident12: i32,
    ident13: u32,
    ident16: i32,
    ident17: u32,
    ident20: u32,
    ident21: f32,
    ident26: u32,
    ident27: f32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident27 = ident28ident0();
    b.ident26 = ident28ident1();
    b.ident21 = ident22ident2(f32(4.0));
    b.ident20 = ident22ident3(u32(3));
    b.ident17 = ident18ident4(u32(4));
    b.ident16 = ident18ident5(i32(3));
    b.ident13 = ident14ident6(u32(2));
    b.ident12 = ident14ident7(i32(1));
}

fn ident28ident0() -> f32 {
    return ident29ident8(f32(2.0));
}

fn ident28ident1() -> u32 {
    return ident29ident9(u32(1));
}

fn ident22ident2(ident23_const: f32) -> f32 {
    var ident23 = ident23_const;
    return ident24ident10(ident23);
}

fn ident22ident3(ident23_const: u32) -> u32 {
    var ident23 = ident23_const;
    return ident24ident11(ident23);
}

fn ident18ident4(ident19_const: u32) -> u32 {
    var ident19 = ident19_const;
    return ident19;
}

fn ident18ident5(ident19_const: i32) -> i32 {
    var ident19 = ident19_const;
    return ident19;
}

fn ident14ident6(ident15_const: u32) -> u32 {
    var ident15 = ident15_const;
    return ident15;
}

fn ident14ident7(ident15_const: i32) -> i32 {
    var ident15 = ident15_const;
    return ident15;
}

fn ident29ident8(ident30_const: f32) -> f32 {
    var ident30 = ident30_const;
    return ident30;
}

fn ident29ident9(ident30_const: u32) -> u32 {
    var ident30 = ident30_const;
    return ident30;
}

fn ident24ident10(ident25_const: f32) -> f32 {
    var ident25 = ident25_const;
    return ident25;
}

fn ident24ident11(ident25_const: u32) -> u32 {
    var ident25 = ident25_const;
    return ident25;
}


// UPDATE SHADER

struct Buffer {
    ident0: i32,
    ident1: u32,
    ident2: i32,
    ident3: u32,
    ident4: u32,
    ident5: f32,
    ident6: u32,
    ident7: f32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}
