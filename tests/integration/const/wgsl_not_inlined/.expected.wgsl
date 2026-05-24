// INIT SHADER

struct Buffer {
    v11: i32,
    v12: i32,
    v21: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

fn _15(_17_const: i32, _19_const: i32) -> i32 {
    var _17 = _17_const;
    var _19 = _19_const;
    return i32(1);
}

fn _23(_25_const: i32, _27_const: i32) -> i32 {
    var _25 = _25_const;
    var _27 = _27_const;
    return i32(2);
}

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.v21 = _23(i32(0), i32(0));
    b.v11 = i32(0);
    b.v12 = _15(b.v11, i32(0));
}


// UPDATE SHADER

struct Buffer {
    v11: i32,
    v12: i32,
    v21: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}

