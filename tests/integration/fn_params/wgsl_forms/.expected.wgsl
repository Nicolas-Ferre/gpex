// INIT SHADER

struct Buffer {
    ident0: i32,
    ident3: i32,
    ident4: i32,
    ident5: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

fn ident1(ident2_const: i32) -> i32 {
    var ident2 = ident2_const;
    return ident2;
}

fn ident6(ident7_const: i32, ident8_const: u32) -> i32 {
    var ident7 = ident7_const;
    var ident8 = ident8_const;
    return ident7;
}

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident5 = ident6(i32(3), u32(4));
    b.ident4 = ident6(i32(2), u32(3));
    b.ident3 = ident6(i32(2), u32(2));
    b.ident0 = ident1(i32(1));
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

