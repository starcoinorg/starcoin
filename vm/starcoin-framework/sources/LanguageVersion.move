address StarcoinFramework {
module LanguageVersion {
    struct LanguageVersion has copy, drop, store {
        major: u64,
    }

    public fun new(version: u64): LanguageVersion {
        LanguageVersion {major: version}
    }

    public fun version(version: &LanguageVersion): u64 {
        version.major
    }

    spec new {
        pragma verify = false;
    }
    spec version {
        pragma verify = false;
    }
}
}