pub struct Node<T> {
    value: T,
    next: *mut Node<T>,
}
