
_default:
  @just -l -u

# Create a tarball of the project
tar:
  ouch c . ram.tar.gz -g

alias b := build
# Build with all features
build:
  cargo build --all-features

alias t := test
# Run tests
test:
  cargo nextest r --all-features
