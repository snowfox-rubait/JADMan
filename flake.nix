{
  description = "JADMan - Advanced Browser-Integrated Download Manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "jadman";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = [ pkgs.pkg-config pkgs.makeWrapper ];
          buildInputs = [ pkgs.openssl ];

          postInstall = ''
            wrapProgram $out/bin/jadm-daemon \
              --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.aria2 pkgs.yt-dlp pkgs.ffmpeg ]}
          '';

          meta = with pkgs.lib; {
            description = "Advanced Browser-Integrated Download Manager daemon and TUI";
            homepage = "https://github.com/snowfox-rubait/JADMan";
            license = licenses.gpl3Only;
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.cargo
            pkgs.rustc
            pkgs.pkg-config
            pkgs.openssl
            pkgs.aria2
            pkgs.yt-dlp
            pkgs.ffmpeg
          ];
        };
      }
    );
}
