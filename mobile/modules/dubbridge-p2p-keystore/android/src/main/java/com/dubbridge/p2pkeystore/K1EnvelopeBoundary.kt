package com.dubbridge.p2pkeystore

import org.json.JSONObject

internal data class NativeHpkeEnvelope(
  val profileVersion: String,
  val keyId: String,
  val encapsulatedKeyBase64: String,
  val ciphertextBase64: String,
  val bindingJson: String,
)

internal data class ValidatedEnvelopeBinding(
  val invitationId: String,
  val viewerId: String,
  val assetId: String,
  val publicationId: String,
  val lineageId: String,
  val authorizationId: String,
  val expiresAtUnix: Long,
)

internal fun HpkeEnvelopeRecord.toNativeEnvelope() = NativeHpkeEnvelope(
  profileVersion = profileVersion,
  keyId = keyId,
  encapsulatedKeyBase64 = encapsulatedKeyBase64,
  ciphertextBase64 = ciphertextBase64,
  bindingJson = bindingJson,
)

internal fun validateEnvelopeBinding(
  envelope: NativeHpkeEnvelope,
  nowUnix: Long,
): ValidatedEnvelopeBinding {
  require(envelope.profileVersion == PROFILE_VERSION) { "Unsupported K1 envelope profile" }
  require(envelope.keyId == KEY_ALIAS) { "K1 envelope targets another device key" }
  require(envelope.bindingJson.isNotBlank()) { "K1 envelope binding is missing" }

  val binding = try {
    JSONObject(envelope.bindingJson)
  } catch (error: Exception) {
    throw IllegalArgumentException("K1 envelope binding is not valid JSON", error)
  }

  require(binding.optString("profile_version") == PROFILE_VERSION) {
    "K1 binding profile does not match the envelope"
  }
  require(binding.optString("device_key_id") == KEY_ALIAS) {
    "K1 binding targets another device key"
  }

  fun requiredId(name: String): String {
    val value = binding.optString(name).trim()
    require(value.isNotEmpty()) { "K1 binding $name is missing" }
    return value
  }

  val expiresAtUnix = binding.optLong("expires_at_unix", Long.MIN_VALUE)
  require(expiresAtUnix > nowUnix) { "K1 envelope binding is expired" }

  return ValidatedEnvelopeBinding(
    invitationId = requiredId("invitation_id"),
    viewerId = requiredId("viewer_id"),
    assetId = requiredId("asset_id"),
    publicationId = requiredId("publication_id"),
    lineageId = requiredId("lineage_id"),
    authorizationId = requiredId("authorization_id"),
    expiresAtUnix = expiresAtUnix,
  )
}
