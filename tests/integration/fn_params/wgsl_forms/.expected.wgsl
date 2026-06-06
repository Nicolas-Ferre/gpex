// INIT SHADER

struct Buffer {
    ident3: i32,
    ident6: i32,
    ident7: i32,
    ident8: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident8 = ident9ident0(u32(4));
    b.ident7 = ident9ident1(u32(3));
    b.ident6 = ident9ident1(u32(2));
    b.ident3 = ident4ident2(i32(1));
}

fn ident9ident0(ident10_const: u32) -> i32 {
    var ident10 = ident10_const;
    return i32(3);
}

fn ident9ident1(ident10_const: u32) -> i32 {
    var ident10 = ident10_const;
    return i32(2);
}

fn ident4ident2(ident5_const: i32) -> i32 {
    var ident5 = ident5_const;
    return ident5;
}


// UPDATE SHADER

struct Buffer {
    ident0: i32,
    ident1: i32,
    ident2: i32,
    ident3: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}

