extern crate error_chain;
extern crate reqwest;
extern crate serde_json;
extern crate std;

#[allow(deprecated)] // WORKAROUND https://github.com/rust-lang-nursery/error-chain/issues/254
error_chain! {
    foreign_links {
        Env(std::env::VarError);
        Http(reqwest::Error);
        InvalidHttpHeader(reqwest::header::InvalidHeaderValue);
        Io(std::io::Error);
        Json(serde_json::Error);
        SystemTime(std::time::SystemTimeError);
    }
}
