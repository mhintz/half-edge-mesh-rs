# Half-Edge Mesh

This is an implementation of the half-edge mesh data structure in Rust. Suggestions and pull requests welcome!

The most notable implementation detail is that links to other parts of the mesh (of which there are many) are `Option<Weak<RefCell<T>>>`. This is in order to allow the references to be initialized to null, then set later during mutation methods. Since a half-edge mesh is a graph with many cycles, and some circular references, the entities require null references at some point during initialization (due to the circular references). I believe this implementation is the only way to construct this kind of structure within Rust's borrowing system, short of using raw pointers and unsafe code everywhere. I suppose that in the far future, I could refactor this project to use raw pointers for performance.

Currently, this data structure is built with mathematical primitives from cgmath. Vectors are `cgmath::Vector3<f32>`, and points are `cgmath::Point3<f32>`. This makes the API somewhat simpler, since it doesn't need to be type-parameterized for primitive types. It would be possible to refactor out this dependency at some point in the future.
