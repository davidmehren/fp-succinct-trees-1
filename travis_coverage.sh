#!/bin/bash
LOCAL="~/.local" # install here to avoid `sudo`
export PATH=$LOCAL/bin:$PATH
if [ ! -f /home/travis/.local/bin/kcov ]; then
    wget https://github.com/SimonKagstrom/kcov/archive/master.tar.gz
    tar xzf master.tar.gz
    mkdir kcov-master/build
    cd kcov-master/build
    cmake -DCMAKE_INSTALL_PREFIX:PATH=$LOCAL ..
    make
    make install
    cd ../..
fi
if [ ! -f /home/travis/.cargo/bin/cargo-kcov ]; then
    cargo install cargo-kcov --force
fi
echo "Cleaning build directory"
rm -rf target
echo "Building project"
cargo build
echo "Running tests"
cargo kcov -v || exit 1
bash <(curl -s https://codecov.io/bash) -s target/cov/kcov-merged -x fix