#! /bin/bash
RUST_LOG=trace \
GST_DEBUG=webrtcbin:6,webrtc:6,nice:6,sctp:6,ERROR:6 \
GST_DEBUG_FILE=gst.log \
cargo run
