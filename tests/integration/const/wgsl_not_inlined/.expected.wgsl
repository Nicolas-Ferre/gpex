// INIT SHADER

struct Buffer {
    ident2: i32,
    ident3: i32,
    ident7: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident7 = ident8ident0(i32(0), i32(0));
    b.ident2 = i32(0);
    b.ident3 = ident4ident1(b.ident2, i32(0));
}

fn ident8ident0(ident9_const: i32, ident10_const: i32) -> i32 {
    var ident9 = ident9_const;
    var ident10 = ident10_const;
    return i32(2);
}

fn ident4ident1(ident5_const: i32, ident6_const: i32) -> i32 {
    var ident5 = ident5_const;
    var ident6 = ident6_const;
    return i32(1);
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
