use error_chain::error_chain;

error_chain! {
    foreign_links {
        Utf8(::std::str::Utf8Error);
        OsErrno(::nix::Error);
        NulError(::std::ffi::NulError);
    }
}
