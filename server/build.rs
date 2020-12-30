fn main() {
    tonic_build::compile_protos("../proto/bouncer.proto").unwrap();
}
