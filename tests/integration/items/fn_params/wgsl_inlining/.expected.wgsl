// INIT SHADER

struct Buffer {
    ident2: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident2 = ident3ident0();
}

fn ident3ident0() -> i32 {
    return ident4ident1();
}

fn ident4ident1() -> i32 {
    return i32(1);
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
