// INIT SHADER

struct Buffer {
    ident6: u32,
    ident7: u32,
    ident8: f32,
    ident11: u32,
    ident12: f32,
    ident16: u32,
    ident17: f32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident17 = ident18ident0(f32(3.0), u32(4));
    b.ident16 = ident18ident1(u32(1), f32(2.0));
    b.ident12 = ident13ident2(f32(3.0), f32(4.0));
    b.ident11 = ident13ident3(u32(1), u32(2));
    b.ident8 = ident9ident4(f32(3.0));
    b.ident7 = ident9ident5(u32(2));
    b.ident6 = ident9ident5(u32(1));
}

fn ident18ident0(ident19_const: f32, ident20_const: u32) -> f32 {
    var ident19 = ident19_const;
    var ident20 = ident20_const;
    return ident19;
}

fn ident18ident1(ident19_const: u32, ident20_const: f32) -> u32 {
    var ident19 = ident19_const;
    var ident20 = ident20_const;
    return ident19;
}

fn ident13ident2(ident14_const: f32, ident15_const: f32) -> f32 {
    var ident14 = ident14_const;
    var ident15 = ident15_const;
    return ident15;
}

fn ident13ident3(ident14_const: u32, ident15_const: u32) -> u32 {
    var ident14 = ident14_const;
    var ident15 = ident15_const;
    return ident15;
}

fn ident9ident4(ident10_const: f32) -> f32 {
    var ident10 = ident10_const;
    return ident10;
}

fn ident9ident5(ident10_const: u32) -> u32 {
    var ident10 = ident10_const;
    return ident10;
}


// UPDATE SHADER

struct Buffer {
    ident0: u32,
    ident1: u32,
    ident2: f32,
    ident3: u32,
    ident4: f32,
    ident5: u32,
    ident6: f32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}
