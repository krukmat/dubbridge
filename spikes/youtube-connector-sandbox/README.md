# YouTube Connector Spike

Status: internal, temporary, not published
Date: 2026-06-03

## Goal

Validate the official, owner-authorized YouTube retrieval path before building
the `PlatformConnector` implementation planned in S3-P3.

## Sources checked

- YouTube Help: download videos that you've uploaded
- Google Account Help: Google Takeout export
- Google for Developers: YouTube Data API `channels.list`
- Google for Developers: YouTube Data API channel implementation guide
- Google for Developers: YouTube Data API OAuth scopes guide

## Result

The official documentation supports a split conclusion:

1. `resolve()` is viable with the YouTube Data API and OAuth 2.0.
2. `download()` is **not** validated as an API-driven server-side byte transfer.

As of 2026-06-03, the official documented byte-retrieval paths for an owner's own
uploads are:

- YouTube Studio per-video download
- Google Takeout export

No official YouTube Data API endpoint was found that returns the media bytes for a
creator's uploaded video to a backend connector.

## Validated v1 resolve path

Use OAuth 2.0 with the minimal read scope for metadata/ownership resolution:

- `https://www.googleapis.com/auth/youtube.readonly`

Owner resolution path:

1. Call `channels.list(part=contentDetails, mine=true)` to resolve the currently
   authenticated channel and its uploads playlist.
2. Treat the channel's uploads playlist as the authoritative set of videos uploaded
   by that authenticated owner.
3. Verify that the requested `SourceRef.external_id` is present in that uploads set.
4. Fetch the target video's metadata through the Data API after ownership is
   confirmed.

Inference from official docs:
- The implementation guide explicitly documents `channels.list(..., mine=true)` for
  the authenticated user's channel and states that `contentDetails` includes the
  playlist IDs for uploaded videos.

## Validated byte-retrieval path

The official documented byte-retrieval mechanisms are user-mediated:

- YouTube Studio allows downloading uploaded videos as MP4 files.
- Google Takeout allows exporting YouTube videos from the owner's account archive.

Implications:

- Studio download is interactive and per-video.
- Studio download has documented limits and failure cases.
- Takeout is archive-oriented, asynchronous, and account-mediated rather than a
  direct connector `download(src, cred, dest)` call.
- Takeout documentation notes that YouTube videos are exported in their original
  format or as MP4 files with H264 video and AAC audio.
- Takeout may require switching into the correct Brand Account context.

## Credential and redaction notes

- Keep owner credentials by reference only; never persist plaintext tokens.
- Redact access tokens, refresh tokens, auth codes, cookie values, and export URLs
  in logs and traces.
- Because the validated byte-retrieval paths are user-mediated, any future
  implementation must also redact Takeout export URLs and email-delivery artifacts.

## Download contract decision

The current official YouTube surface does **not** validate the `PlatformConnector`
`download()` contract as planned for S3-P3.

That means one of these replans is required before implementation:

1. Change the product flow to ingest a user-mediated export artifact
   (Studio download or Takeout archive) instead of backend-driven connector
   download.
2. Defer the YouTube connector and prioritize a platform with an official
   server-to-server media download API.
3. Introduce a different, explicitly approved acquisition path backed by official
   provider documentation.

## Practical recommendation

Do not implement S3-P3 as currently worded. Replan it first.
