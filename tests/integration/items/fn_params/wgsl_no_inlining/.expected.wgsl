// INIT SHADER

struct Buffer {
    ident1: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident1 = ident2ident0(i32(1));
}

fn ident2ident0(ident3_const: i32) -> i32 {
    var ident3 = ident3_const;
    return ident3;
}


// UPDATE SHADER

struct Buffer {
    ident0: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}
