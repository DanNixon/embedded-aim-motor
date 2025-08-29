{pkgs, ...}: {
  packages = with pkgs; [
    # Code formatting tools
    treefmt
    alejandra
    mdl
    rustfmt

    # Rust toolchain
    rustup

    # Release tools
    release-plz

    # Tools to run examples
    probe-rs
  ];
}
