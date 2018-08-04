Same
====

A crate for comparing references in generic way.

About
-----

This crate provides mainly the trait `Some` which can be used on *shared*
reference types. (`&T`, `Rc`, `Arc`). It enables the users to test identity
of objects. It's analogous to `PartialEq`, which tests *equality* instead.

Additionally, this crate provides `RefHash` trait, which is used for hashing
references and `RefCmp` wrapper struct, which implements `Eq`, `PartialEq`
and `Hash` by delegating to `Same` and `RefHash` traits. This is mainly
useful if one wants to store objects in `HashSet` or similar data structure,
with.

This crate is `no_std`-compatible.

License
-------

MITNFA
