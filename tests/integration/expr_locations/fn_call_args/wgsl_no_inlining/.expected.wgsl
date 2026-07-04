// INIT SHADER

struct Buffer {
    ident1: i32,
    ident2: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident1 = i32(0);
    b.ident2 = ident3ident0(b.ident1, i32(0));
}

fn ident3ident0(ident4_const: i32, ident5_const: i32) -> i32 {
    var ident4 = ident4_const;
    var ident5 = ident5_const;
    return i32(1);
}


// UPDATE SHADER

struct Buffer {
    ident0: i32,
    ident1: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}
