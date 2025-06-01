{
  description = "A flake for deckgym-core cli";

  inputs = {
    # Add the nixpkgs flake for package dependencies
    nixpkgs.url = "github:NixOS/nixpkgs";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";  # Adjust for your system (e.g., "aarch64-linux" for ARM)
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
        pname = "deckgym-core-cli";
        version = "0.1.0";

        # Set your source directory (can be a relative path to Cargo.toml)
        src = ./.;

        cargoHash = "sha256-CFfKWdMFafWjxIsEiRDBiT5mTlskV27GK/ERuhqRBuU=";

        nativeBuildInputs = [ pkgs.pkg-config ];

        # Optional: Specify any additional dependencies
        buildInputs = [ ];

        # Optional: Enable crate features (if needed)
        cargoFeatures = [ "--verbose" "--release" "--jobs 16" ];
      };

      # This allows you to run the package as a flake app
      apps.default = {
        type = "app";
        program = "${self.packages.${system}.default}/bin/deckgym-core-cli";
      };
    };
}

