// INIT SHADER

struct Buffer {
    ident0: vec2<u32>,
    ident1: vec2<u32>,
    ident2: vec2<u32>,
    ident3: vec2<u32>,
    ident4: vec2<u32>
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.ident4 = vec2<u32>(0, 0);
    b.ident3 = vec2<u32>(0, 4);
    b.ident2 = vec2<u32>(0, 2);
    b.ident1 = vec2<u32>(0, 1);
    b.ident0 = vec2<u32>(0, 3);
}


// UPDATE SHADER

struct Buffer {
    ident0: vec2<u32>,
    ident1: vec2<u32>,
    ident2: vec2<u32>,
    ident3: vec2<u32>,
    ident4: vec2<u32>
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}
