{
    lib,
    rustPlatform,
    lvm2
}:

rustPlatform.buildRustPackage (finalAttrs: {
    pname = "lvm-homepage";
    version = "0.1.0";

    src = ./.;

    postPatch = ''
      substituteInPlace src/main.rs \
        --replace-fail '"lvs",//command' '"${lvm2.bin}/bin/lvs",' \
        --replace-fail '"vgs",//command' '"${lvm2.bin}/bin/vgs",' \
        --replace-fail '"pvs",//command' '"${lvm2.bin}/bin/pvs",'
    '';

    cargoHash = "sha256-dkIDZXinSYncAouEcCnr8T5AQAhksyKrTmhvAk+qA44=";

    meta = {
      description = "Service to query lvs, pvs and vgs, and provide that as an api for homepage-dashboard";
      homepage = "https://github.com/nofishleft/lvm-homepage";
      license = lib.licenses.sustainableUse;
    };
})