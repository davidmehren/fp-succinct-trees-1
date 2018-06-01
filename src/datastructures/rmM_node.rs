use std::fmt::Debug;

pub struct rmM_node {
    e : i32,
    m : i32,
    n : u64,
    M : i32,
    starts : u64,
    ends : u64,
}

impl Debug for rmM_node {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}, {}, {}, {}, {}, {})", self.e, self.m, self.n, self.M, self.starts, self.ends)
    }
}