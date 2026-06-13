// INIT SHADER

struct Buffer {
    ident8: i32,
    ident9: u32,
    ident12: u32,
    ident13: f32,
    ident18: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident18 = ident19ident0();
    b.ident13 = ident14ident1(f32(4.0));
    b.ident12 = ident14ident2(u32(3));
    b.ident9 = ident10ident3(u32(2));
    b.ident8 = ident10ident4(i32(1));
}

fn ident19ident0() -> i32 {
    return ident20ident5(i32(1));
}

fn ident14ident1(ident15_const: f32) -> f32 {
    var ident15 = ident15_const;
    return ident16ident6(ident15);
}

fn ident14ident2(ident15_const: u32) -> u32 {
    var ident15 = ident15_const;
    return ident16ident7(ident15);
}

fn ident10ident3(ident11_const: u32) -> u32 {
    var ident11 = ident11_const;
    return ident11;
}

fn ident10ident4(ident11_const: i32) -> i32 {
    var ident11 = ident11_const;
    return ident11;
}

fn ident20ident5(ident21_const: i32) -> i32 {
    var ident21 = ident21_const;
    return i32(1);
}

fn ident16ident6(ident17_const: f32) -> f32 {
    var ident17 = ident17_const;
    return ident17;
}

fn ident16ident7(ident17_const: u32) -> u32 {
    var ident17 = ident17_const;
    return ident17;
}


// UPDATE SHADER

struct Buffer {
    ident0: i32,
    ident1: u32,
    ident2: u32,
    ident3: f32,
    ident4: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}
