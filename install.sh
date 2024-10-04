#!/bin/bash

cargo build --release --quiet
cp target/release/ozy ~/.ozy/bin
