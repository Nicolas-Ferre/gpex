// INIT SHADER

struct Buffer {
    v5: i32,
    v6: i32,
    v15: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

fn _9(_11_const: i32, _13_const: i32) -> i32 {
    var _11 = _11_const;
    var _13 = _13_const;
    return i32(1);
}

fn _17(_19_const: i32, _21_const: i32) -> i32 {
    var _19 = _19_const;
    var _21 = _21_const;
    return i32(2);
}

@compute @workgroup_size(1, 1, 1)
fn main() {
    b.v15 = _17(i32(0), i32(0));
    b.v5 = i32(0);
    b.v6 = _9(b.v5, i32(0));
}


// UPDATE SHADER

struct Buffer {
    v5: i32,
    v6: i32,
    v15: i32
}

@group(0) @binding(0)
var<storage, read_write> b: Buffer;

@compute @workgroup_size(1, 1, 1)
fn main() {

}

