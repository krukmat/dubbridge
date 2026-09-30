# S-230-T6d.3 — Controlled reuse, resize and IaC adoption

Date: 2026-09-29
Branch: `main`
Status: SUPERSEDED — NOT EXECUTED

## Scope

Reuse Droplet `144322723` in `fra1` as the DubBridge POC host after the
T6d.2a preservation gate.

The operation is deliberately split:

1. verify the WordPress backup and exact Droplet identity;
2. perform a CPU/RAM-only resize to `s-2vcpu-4gb`;
3. power the Droplet back on and verify its IP/size;
4. import the existing Droplet and DNS record into OpenTofu state.

No firewall, database or media Space is created in this task.

## Commands

Dry preflight:

```bash
make do-reuse-resize
```

Execute resize:

```bash
DO_T6D3_EXECUTE_RESIZE=1 make do-reuse-resize
```

Then IaC adoption:

```bash
make do-import-reused-base
```

## Safety properties

- exact Droplet ID: `144322723`;
- exact public IP: `46.101.217.151`;
- runtime region: `fra1`;
- backup checksums must verify before resize;
- only current size `s-1vcpu-2gb` may transition to `s-2vcpu-4gb`;
- disk resize is disabled;
- imported historical image/SSH-key attributes cannot trigger a rebuild;
- create/apply is not part of T6d.3.


## Hold disposition

Owner paused execution on 2026-09-29 before any resize or import.

No T6d.3 cloud mutation has occurred. The WordPress preservation evidence from
T6d.2a remains the recovery baseline. Resume from the guarded preflight before
enabling the resize execution gate.


## Superseded disposition

On 2026-09-30 the owner selected a lower-cost Contabo + Cloudflare R2 runtime baseline. This Digital Ocean resize/import path is therefore superseded and remains unexecuted. The T6d.2a WordPress preservation package remains valid input for the explicit WordPress migration task `T6d.C3a` on the new host.
