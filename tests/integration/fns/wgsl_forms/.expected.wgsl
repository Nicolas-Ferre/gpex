// INIT SHADER

struct Buffer {
    ident6: i32,
    ident7: u32,
    ident10: i32,
    ident11: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident11 = ident12ident0(f32(4.0));
    b.ident10 = ident12ident1(u32(3));
    b.ident7 = ident8ident2(u32(2));
    b.ident6 = ident8ident3(i32(1));
}

fn ident12ident0(ident13_const: f32) -> i32 {
    var ident13 = ident13_const;
    return ident14ident4(ident13);
}

fn ident12ident1(ident13_const: u32) -> i32 {
    var ident13 = ident13_const;
    return ident14ident5(ident13);
}

fn ident8ident2(ident9_const: u32) -> u32 {
    var ident9 = ident9_const;
    return ident9;
}

fn ident8ident3(ident9_const: i32) -> i32 {
    var ident9 = ident9_const;
    return ident9;
}

fn ident14ident4(ident15_const: f32) -> i32 {
    var ident15 = ident15_const;
    return i32(3);
}

fn ident14ident5(ident15_const: u32) -> i32 {
    var ident15 = ident15_const;
    return i32(3);
}


// UPDATE SHADER

struct Buffer {
    ident0: i32,
    ident1: u32,
    ident2: i32,
    ident3: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}
