---
type: Playbook
status: design-candidate
date: 2026-11-25
---

# D4 — Network and shared-folder boundary

## D4.1a alternatives

Apple TN3165 (https://developer.apple.com/documentation/technotes/tn3165-packet-filter-is-not-api)
explains PF is not an application API and recommends Network Extension for
TCP/UDP content filtering. The macOS 26.7 pf.conf manual documents user
matching for TCP/UDP sockets, not all IPC or all protocols.
Reference: https://man.freebsd.org/cgi/man.cgi?manpath=macOS+26.7&query=pf.conf&sektion=5.
Do not treat a PF UID anchor as full sandbox proof, especially with system
DNS/proxy services, preexisting states, loopback exclusions and delegated
connections. No automatic PF modification: the Mac has other workloads.
Network Extension is the supported platform direction, but entails
entitlements, signing, installation, policy approval and host flow testing.
Application allowlists, stripped proxy env and sandbox-exec alone are not
a non-bypassable OS boundary. A VM would change the approved architecture
and consume resources; do not add one implicitly.

Decision: no new firewall or system extension in the current synthetic
gate. Specify a deny-by-default policy, measure gaps locally, and defer
enforcement choice until the host can prove a compatible narrow mechanism.
D4.1c BLOCKED for actual network containment. Do not claim a GO or reduce
the required boundary to a cooperative application check.

## D4.1b intended policy

| Identity | Allowed | Denied by intended boundary |
|---|---|---|
| Agent 472 | bounded Unix broker; exact approved local model transport if needed | HAA endpoint, private credentials, other local/external destinations |
| Verifier 473 | read-only export and own temporary build outputs; no network required for synthetic tests | all TCP/UDP, DNS/relay paths, HAA, model and owner services |
| Executor 471 | local HAA endpoint with protected role clients; protected broker | arbitrary model/repository commands and unrestricted worker-controlled destinations |

Actual production endpoints are not assumed. No cloud model, remote Git
push, package fetch or arbitrary network-enabled test is admitted by this
synthetic profile. Model transports, external dependency needs and remote
effects require explicit separate authorization and a proven boundary.
No proxies, DNS relays, IPv6, loopback, inherited sockets, system services
or Unix IPC exceptions by accident. Validate real host controls on both
families and protocols; never attempt private-file or external-data tests.
HAA client proxy/redirect rejection is defense in depth, not egress isolation.

## D4.2 sharing

The owner approved dedicated UID 471/472/473; macOS automatically assigned
system groups. Do not delete system-managed group memberships or inspect
actual user documents. Owner creates synthetic canaries with representative
ACL and mode only, including representative shared-folder access rules.
Compare effective read/write/rename/delete from each identity. A readable
public canary is not evidence of exposure of private data, and a denied
canary does not prove the entire host is isolated. Record unexpected
access and review share configuration without reading private content.
Use scripts/haa/sharing_probe.py only on the exact synthetic tree.
The owner must supply expected access policy; unexpected access BLOCKED.
No owner home scanning, SMB connection, network transmission or permission
mutation outside the synthetic fixture.

## D4.1c deployment gate

Before actual enforcement, select and certify an OS-supported approach.
If no such narrow mechanism can be installed under the owner-approved
resource/privilege constraints, remain NO_GO. A negative local TCP probe
does not establish all traffic/relay containment. Do not introduce more
hardware or automatic privileged policy changes. Preserve G1 evidence.
