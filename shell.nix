with import <nixpkgs> {};
mkShell {
  buildInputs = [
    gdb   # required for rust-gdb
    gh
    clang # required to compile enet-sys
    cmake # required to compile enet-sys
    rustup
    rust-analyzer
    yamllint
  ];
  # needed for enet-sys crate so bindgen can find libclang.so
  LIBCLANG_PATH="${llvmPackages.libclang.lib}/lib";
}
