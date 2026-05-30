// INIT SHADER

struct Buffer {
    ident0: i32,
    ident1: vec2<u32>,
    ident2: i32,
    ident3: i32,
    ident8: i32,
    ident9: i32,
    ident10: i32,
    ident11: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

fn ident4(ident5_const: i32) -> i32 {
    var ident5 = ident5_const;
    return ident6(ident5);
}

fn ident6(ident7_const: i32) -> i32 {
    var ident7 = ident7_const;
    return ident7;
}

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident11 = i32(6);
    b.ident10 = i32(5);
    b.ident9 = i32(4);
    b.ident8 = i32(3);
    b.ident3 = ident4(i32(7));
    b.ident2 = i32(2);
    b.ident1 = vec2<u32>(0, 1);
    b.ident0 = i32(1);
}


// UPDATE SHADER

struct Buffer {
    ident0: i32,
    ident1: vec2<u32>,
    ident2: i32,
    ident3: i32,
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

