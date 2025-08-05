#!/bin/bash

set -x

# Global vars
install_dir='/usr/local/bin'
install_config_dir='/data'

# Build rust bin file
cargo build

# Install upload upyun bin
sudo cp $(pwd)/../target/debug/upyun_tool $install_dir
sudo cp $(pwd)/.upyun_pass $install_config_dir

