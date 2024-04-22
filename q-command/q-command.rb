class QCommand < Formula
  desc "A command-line tool for executing MacOSX bash scripts"
  url "https://alimaslax.com"
  sha256 "0c0544d03b18cc89b114c7ee8b3be668dec72fe1c0155584473116808c22becd"
  version "1.0.0" # <--- Add this line

  depends_on :macos

  def install
    # Build the Rust project
    system "cargo", "build", "--release"

    # Install the binary
    bin.install "target/release/q"
  end
end