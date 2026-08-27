export const BARE_WORKLET_FILENAME = "/bare-worklet.js";

/**
 * This source is intentionally dependency-free so the native Worklet/IPC
 * boundary stays available before P2P dependencies are introduced.
 */
export const BARE_WORKLET_SOURCE = `
const { IPC } = BareKit

function send(message) {
  try {
    IPC.write(Buffer.from(JSON.stringify(message)))
  } catch (_) {
    // The host may have terminated while an event was in flight. Do not throw
    // from the worklet handler: an unhandled worklet exception aborts the host.
  }
}

function error(id, code, message) {
  send({ type: 'error', id, code, message })
}

IPC.on('data', (data) => {
  let request

  try {
    request = JSON.parse(data.toString())
  } catch (_) {
    error(null, 'INVALID_REQUEST', 'Bare request is not valid JSON')
    return
  }

  if (!request || request.type !== 'request' || typeof request.id !== 'string') {
    error(null, 'INVALID_REQUEST', 'Bare request has an invalid envelope')
    return
  }

  try {
    switch (request.command) {
      case 'initialize':
        send({ type: 'result', id: request.id, value: 'ready' })
        return
      case 'ping':
        send({ type: 'result', id: request.id, value: 'pong' })
        return
      case 'shutdown':
        send({ type: 'result', id: request.id, value: 'stopped' })
        return
      default:
        error(request.id, 'UNKNOWN_COMMAND', 'Bare command is not supported')
    }
  } catch (_) {
    error(request.id, 'WORKLET_FAILURE', 'Bare worklet command failed')
  }
})
`;
