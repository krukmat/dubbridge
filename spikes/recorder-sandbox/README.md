# Recorder Sandbox Spike

Internal-only evidence for S3 Task T0c. Do not publish.

## Purpose

Validate one FFmpeg recording output contract using a synthetic local source before
building `crates/recorder`.

## Environment

- Host FFmpeg: `8.1`
- Synthetic inputs:
  - `lavfi:testsrc2=size=320x240:rate=30`
  - `lavfi:sine=frequency=1000:sample_rate=48000`

## Commands exercised

### Fixed-duration HLS fMP4 staging

```text
ffmpeg -y -hide_banner -loglevel error \
  -f lavfi -re -i testsrc2=size=320x240:rate=30 \
  -f lavfi -re -i sine=frequency=1000:sample_rate=48000 \
  -t 7 \
  -c:v libx264 -preset veryfast -g 60 -sc_threshold 0 \
  -c:a aac \
  -f hls -hls_time 2 -hls_playlist_type event \
  -hls_segment_type fmp4 \
  -hls_fmp4_init_filename init.mp4 \
  -hls_segment_filename 'spikes/recorder-sandbox/hls-fmp4-fixed/seg_%03d.m4s' \
  spikes/recorder-sandbox/hls-fmp4-fixed/session.m3u8
```

Observed output:

- `init.mp4`
- `session.m3u8`
- `seg_000.m4s` .. `seg_003.m4s`
- clean `#EXT-X-ENDLIST`

### Whole-session remux after finalized manifest

```text
ffmpeg -y -hide_banner -loglevel error \
  -i spikes/recorder-sandbox/hls-fmp4-fixed/session.m3u8 \
  -c copy \
  spikes/recorder-sandbox/hls-fmp4-fixed/assembled.mp4
```

Observed result:

- remux succeeded
- assembled MP4 created from the finalized manifest

### Graceful stop (`q`)

The same HLS fMP4 command was run without `-t`, then stopped by writing `q` to
stdin.

Observed result:

- FFmpeg finalized the last segment
- manifest was updated with `#EXT-X-ENDLIST`
- remux from the manifest to one assembled MP4 succeeded

### Hard kill

The same command was run and then terminated with `kill -9`.

Observed result:

- completed `.m4s` segments remained on disk
- `session.m3u8` existed but had no `#EXT-X-ENDLIST`
- direct remux from the open manifest could block waiting for more input

## Decision supported by the spike

V1 uses:

- local HLS fMP4 segmented staging during capture
- one assembled MP4 as the asset boundary
- bridge only after clean stop or an explicit future recovery path

Rejected for v1:

- per-segment assets
- manifest-backed assets
- automatic bridge from crash-open manifests
