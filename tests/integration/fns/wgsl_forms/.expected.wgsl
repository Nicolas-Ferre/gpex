// INIT SHADER

struct Buffer {
    ident2: i32,
    ident3: u32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident3 = ident4ident0(u32(2));
    b.ident2 = ident4ident1(i32(1));
}

fn ident4ident0(ident5_const: u32) -> u32 {
    var ident5 = ident5_const;
    return ident5;
}

fn ident4ident1(ident5_const: i32) -> i32 {
    var ident5 = ident5_const;
    return ident5;
}


// UPDATE SHADER

struct Buffer {
    ident0: i32,
    ident1: u32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}
