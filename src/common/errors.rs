#[derive(Fail, Debug)]
#[fail(display = "Supplied an invalid index")]
pub struct IndexOutOfBoundsError;

#[derive(Fail, Debug)]
pub enum NodeError {
    #[fail(display = "The supplied index does not reference a node.")]
    NotANodeError,
    #[fail(display = "The supplied index does not reference a leaf.")]
    NotALeafError,
    #[fail(display = "The supplied index does not reference a node with leafs.")]
    NotAParentError,
}
