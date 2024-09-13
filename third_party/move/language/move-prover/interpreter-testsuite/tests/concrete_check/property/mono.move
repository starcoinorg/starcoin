address 0x2 {
module Base {
    struct B has key {}

    public fun BASE_ADDR(): address {
        @0x1
    }

    public fun put_b(s: &signer) {
        move_to(s, B {});
    }

    spec module {
        fun has_b(): bool {
            exists<B>(BASE_ADDR())
        }
    }
}

module Test {
    use 0x2::Base;

    struct R<T: store> has key {
         f: T,
    }

    public fun put_r<T: store>(s: &signer, v: T) {
        Base::put_b(s);
        move_to(s, R { f: v });
    }

    #[test(s=@0x2)]
    public fun check_0x2_pass(s: &signer) {
        put_r(s, true);
    }

    #[test(s=@0x1)]
    public fun check_0x1_fail(s: &signer) {
        put_r(s, true);
    }

    spec module {
        fun has_r<T>(): bool {
            exists<R<T>>(Base::BASE_ADDR())
        }
    }

    spec module {
        invariant<T> update
            (Base::has_b() && has_r<T>()) ==> old(has_r<T>());
    }
}
}
