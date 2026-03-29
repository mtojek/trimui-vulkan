#!/bin/sh

cd "$(dirname "$0")/.demos-rust"
./vulkancube_sdl2 > ./vulkancube_sdl2.log 2>&1
