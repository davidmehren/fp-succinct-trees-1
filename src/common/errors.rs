#[derive(Fail, Debug)]
#[fail(display = "Supplied an invalid index")]
pub struct IndexOutOfBoundsError;

#[derive(Fail, Debug, PartialEq)]
pub enum NodeError {
    #[fail(display = "The supplied index does not reference a node.")]
    NotANodeError,
    #[fail(display = "The supplied index does not reference a leaf.")]
    NotALeafError,
    #[fail(display = "The supplied index does not reference a node with leafs.")]
    NotAParentError,
    #[fail(display = "The supplied index references the root node.")]
    RootNodeError,
    #[fail(display = "The supplied index does not reference a node with a parent.")]
    HasNoParentError,
    #[fail(display = "The supplied index does not reference a node with a next sibling.")]
    HasNoFurtherSiblingsError,
}
