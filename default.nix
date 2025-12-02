{
    lib,
    rustPlatform,
}:

rustPlatform.buildRustPackage (finalAttrs: {
    pname = "lvm-homepage";
    version = "0.1.0";

    src = ./;

    cargoHash = lib.fakeHash;
})