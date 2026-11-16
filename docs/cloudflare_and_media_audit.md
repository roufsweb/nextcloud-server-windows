# Nextcloud Server: Cloudflare Tunnel & Media Streaming Audit

This document provides an exhaustive architectural audit of Nextcloud features, community packages, Cloudflare Tunnel constraints, and media streaming optimizations for home-server environments.

---

## 1. Cloudflare Tunnel Capability & Constraint Matrix

Running Nextcloud behind a Cloudflare Tunnel (`cloudflared`) provides instant, zero-port-forwarding HTTPS access worldwide. However, Cloudflare imposes specific edge constraints that require architectural accommodations:

| Feature / App | Works over Tunnel? | Constraint / Limitation | Engineering Solution |
| :--- | :---: | :--- | :--- |
| **File Sync & WebDAV** | ✅ **YES** | 100 MB HTTP payload limit on Free/Pro plans | **Nextcloud Chunking**: Nextcloud splits all uploads into 20 MB chunks (`'chunk_size' => 20971520`). Files up to 100GB+ upload smoothly without hitting the 100MB limit. |
| **Nextcloud Memories** | ✅ **YES** | Slow home upload chokes on raw 4K/8K media | **Adaptive Transcoding & Previews**: Nextcloud pre-generates 1080p/720p H.264/WebP previews. Raw 100Mbps camera files stream at 2–4 Mbps. |
| **Nextcloud Photos** | ✅ **YES** | On-the-fly preview generation causes 524 timeouts | **Background Pre-Generation**: Previews pre-compiled via background cron (`occ preview:pre-generate`). |
| **Nextcloud Talk** | ⚠️ **PARTIAL** | WebSockets work for signaling; UDP media blocked | **Signaling over Tunnel**: Chat, presence, and WebSockets work. Voice/video calls require a public STUN server (`stun:stun.l.google.com:19302`) or external TURN. |
| **Nextcloud Office (Collabora)** | ✅ **YES** | WOPI token and iframe cross-origin requirements | **Reverse Proxy Overwrite**: `'overwriteprotocol' => 'https'` and `'trusted_proxies'` configured so WOPI traffic validates correctly. |
| **Contacts & Calendar (CalDAV/CardDAV)** | ✅ **YES** | iOS / Android redirect path requirements | **Caddy Redirects**: Caddyfile redirects `/.well-known/caldav` and `/.well-known/carddav` to `/remote.php/dav`. |
| **Live Video Playback** | ✅ **YES** | Cloudflare CDN terms prohibit raw video caching | **Private Cache-Control**: Set `Cache-Control: private, no-transform` on media endpoints so traffic streams without triggering CDN caching blocks. |
| **SMB / Local Network Mounts** | ❌ **NO (Direct)** | Cloudflare Tunnel is HTTP/HTTPS only | **External Storage App**: Direct Windows drives (`D:\`, `E:\`) must be mounted via Nextcloud's `files_external` (Local Storage) on the host machine. |

---

## 2. Low-Upload Home Server Optimization

Home internet connections are asymmetric (e.g. 100–300 Mbps download, but only 10–30 Mbps upload). Serving raw 4K/60fps video (which bitrate exceeds 80–120 Mbps) directly from a home server will buffer indefinitely over remote connections.

### Architectural Solutions:
1. **Nextcloud Chunked Uploads (20 MB Chunks)**:
   - Configured in Nextcloud config: `'chunk_size' => '20971520'`.
   - Desktop and Mobile clients slice massive transfers (RAW footage, drone archives) into 20MB slices. If connection drops, only the current 20MB chunk is retried, not the whole file.
2. **Fast Response Streaming in Caddy**:
   - `flush_interval -1`: Caddy flushes FastCGI output immediately to Cloudflare, preventing the 100-second idle connection timeout (HTTP 524).
3. **Adaptive Video Streaming via Nextcloud Memories (`go-vod`)**:
   - Transcodes heavy 4K/5.4K H.265/HEVC video into lightweight 720p/1080p H.264 on-the-fly or on-demand, streaming at 2.5–4 Mbps suitable for mobile 4G and home connections.
4. **Pre-Generated WebP Thumbnails**:
   - Generates multi-resolution WebP thumbnails (`preview_max_x => 2048`, `preview_max_y => 2048`). Loading photo albums consumes kilobytes instead of downloading megabytes of raw images.

---

## 3. Modern Camera & Mobile Formats Compatibility

Modern smartphones (iPhones) and professional cameras/drones use specialized codecs and raw containers:

| Format Category | File Extensions | Handling Engine | Nextcloud Capability |
| :--- | :--- | :--- | :--- |
| **iPhone Photos** | `.HEIC`, `.HEIF`, `.AVCI` | ImageMagick + `libheif` | Full thumbnail generation, EXIF metadata extraction, timeline sorting. |
| **iPhone Live Photos** | `.HEIC` + `.MOV` pair | Memories App (`livephoto`) | Merges photo + motion clip into interactive Live Photo in web UI and mobile app. |
| **Apple ProRAW & DNG** | `.DNG` | ImageMagick + `libraw` | Decodes 12-bit/14-bit Bayer raw data into color-corrected WebP previews. |
| **iPhone ProRes Video** | `.MOV` (ProRes 422 / HQ) | FFmpeg / `go-vod` | Extracts keyframes, transcodes high-bitrate ProRes into H.264 streamable proxy. |
| **Drone Video (DJI / Autel)** | `.MP4`, `.MOV` (D-Log, D-Log M) | FFmpeg / `go-vod` | Streams video previews, generates timeline scrubber strips. |
| **Pro Camera Log Video** | Sony S-Log2/3, Canon C-Log, Nikon N-Log | FFmpeg | Generates playable preview proxies; original uncompressed files preserved intact for download. |
| **Camera RAW Stills** | `.CR2`, `.CR3`, `.NEF`, `.ARW`, `.RAF` | ImageMagick + dcraw/libraw | Extracts embedded JPEG preview without decoding full 50MB raw buffer. |

---

## 4. Top Recommended Nextcloud Community Apps

To achieve the best media, office, and backup experience, the following apps are recommended for installation via `occ app:install`:

1. **`memories`** *(Priority 1)*: High-performance Google Photos replacement with timeline, map view, face recognition, and Live Photos support.
2. **`files_external`** *(Pre-enabled)*: Allows mounting local drives (`D:\`, `E:\`) directly with full Windows NTFS permissions.
3. **`richdocuments` (Nextcloud Office)**: Web-based editing for DOCX, XLSX, and PPTX via Collabora Online.
4. **`spreed` (Nextcloud Talk)**: Secure instant messaging and audio/video calling.
5. **`previewgenerator`**: Background cron worker that pre-generates thumbnail pyramids.
6. **`audioplayer` / `music`**: Web-based streaming player for FLAC, MP3, AAC, and ALAC audio libraries.
7. **`twofactor_webauthn`**: Hardware key (YubiKey / Windows Hello) two-factor authentication.
8. **`tasks` & `deck`**: Kanban-style project management and task synchronization.
