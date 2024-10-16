/// A ring-shaped container that can hold any type, indexed from 0
/// The capacity is fixed at creation time, and the accessible index is constantly growing
module starcoin_framework::ring {

    use std::error;
    use std::option;
    use std::vector;

    /// The index into the vector is out of bounds
    const ERROR_RING_INDEX_OUT_OF_BOUNDS: u64 = 101;

    struct Ring<Element> has store {
        data: vector<option::Option<Element>>,
        insertion_index: u64,
        external_index: u64
    }

    /// Create a Ring with capacity.
    public fun create_with_capacity<Element>(len: u64): Ring<Element> {
        let data = vector::empty<option::Option<Element>>();
        let i = 0;
        while (i < len) {
            vector::push_back(&mut data, option::none<Element>());
            i = i + 1;
        };
        Ring {
            data: data,
            insertion_index: 0,
            external_index: 0,
        }
    }

    spec create_with_capacity {
        pragma intrinsic = true;
    }

    ///is Ring full
    public fun is_full<Element>(r: &Ring<Element>): bool {
        option::is_some(vector::borrow(&r.data, r.insertion_index))
    }

    spec is_full {
        pragma intrinsic = true;
    }

    ///Return the capacity of the Ring.
    public fun capacity<Element>(r: &Ring<Element>): u64 {
        vector::length(&r.data)
    }

    spec capacity {}

    /// Add element `e` to the insertion_index of the Ring `r`.
    public fun push<Element>(r: &mut Ring<Element>, e: Element): option::Option<Element> {
        let op_e = vector::borrow_mut<option::Option<Element>>(&mut r.data, r.insertion_index);
        let res = if (option::is_none<Element>(op_e)) {
            option::fill(op_e, e);
            option::none<Element>()
        }else {
            option::some<Element>(option::swap(op_e, e))
        };
        r.insertion_index = (r.insertion_index + 1) % vector::length(&r.data);
        r.external_index = r.external_index + 1;
        res
    }

    spec push {
        pragma intrinsic = true;
    }

    /// Return a reference to the `i`th element in the Ring `r`.
    public fun borrow<Element>(r: & Ring<Element>, i: u64): &option::Option<Element> {
        let len = capacity<Element>(r);
        if (r.external_index > len - 1) {
            assert!(
                i >= r.external_index - len && i < r.external_index,
                error::invalid_argument(ERROR_RING_INDEX_OUT_OF_BOUNDS)
            );
            vector::borrow(&r.data, i % len)
        }else {
            assert!(i < len, error::invalid_argument(ERROR_RING_INDEX_OUT_OF_BOUNDS));
            vector::borrow(&r.data, i)
        }
    }

    spec borrow {
        pragma intrinsic = true;
    }

    /// Return a mutable reference to the `i`th element in the Ring `r`.
    public fun borrow_mut<Element>(r: &mut Ring<Element>, i: u64): &mut option::Option<Element> {
        let len = capacity<Element>(r);
        if (r.external_index > len - 1) {
            assert!(
                i >= r.external_index - len && i < r.external_index,
                error::invalid_argument(ERROR_RING_INDEX_OUT_OF_BOUNDS)
            );
            vector::borrow_mut(&mut r.data, i % len)
        }else {
            assert!(i < len, error::invalid_argument(ERROR_RING_INDEX_OUT_OF_BOUNDS));
            vector::borrow_mut(&mut r.data, i)
        }
    }

    /// Return `option::Option<u64>` if `e` is in the Ring `r` at index `i`.
    /// Otherwise, returns `option::none<u64>`.
    public fun index_of<Element>(r: &Ring<Element>, e: &Element): option::Option<u64> {
        let i = 0;
        let len = capacity<Element>(r);
        while (i < len) {
            if (option::borrow(vector::borrow(&r.data, i)) == e) return option::some(i + r.external_index - len);
            i = i + 1;
        };
        option::none<u64>()
    }

    spec index_of {
        pragma intrinsic = true;
    }

    /// Destroy the Ring `r`.
    /// Returns the vector<Element> saved by ring
    public fun destroy<Element>(r: Ring<Element>): vector<Element> {
        let Ring {
            data: data,
            insertion_index: _,
            external_index: _,
        } = r ;
        let len = vector::length(&data);
        let i = 0;
        let vec = vector::empty<Element>();
        while (i < len) {
            let op_e = vector::pop_back(&mut data);
            if (option::is_some(&op_e)) {
                vector::push_back(&mut vec, option::destroy_some(op_e))
            }else {
                option::destroy_none(op_e)
            };
            i = i + 1;
        };
        vector::destroy_empty(data);
        vec
    }
}
