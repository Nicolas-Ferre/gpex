// INIT SHADER

struct Buffer {
    ident0: i32,
    ident1: i32,
    ident5: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

fn ident2(ident3_const: i32, ident4_const: i32) -> i32 {
    var ident3 = ident3_const;
    var ident4 = ident4_const;
    return i32(1);
}

fn ident6(ident7_const: i32, ident8_const: i32) -> i32 {
    var ident7 = ident7_const;
    var ident8 = ident8_const;
    return i32(2);
}

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident5 = ident6(i32(0), i32(0));
    b.ident0 = i32(0);
    b.ident1 = ident2(b.ident0, i32(0));
}


// UPDATE SHADER

struct Buffer {
    ident0: i32,
    ident1: i32,
    ident2: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}

