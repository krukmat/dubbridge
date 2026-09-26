package com.dubbridge.p2pkeystore

import android.security.keystore.KeyProperties
import android.util.Base64
import androidx.test.ext.junit.runners.AndroidJUnit4
import java.math.BigInteger
import java.nio.charset.StandardCharsets
import java.security.KeyPairGenerator
import java.security.KeyStore
import java.security.interfaces.ECPublicKey
import java.security.spec.ECGenParameterSpec
import javax.crypto.Cipher
import javax.crypto.KeyAgreement
import javax.crypto.Mac
import javax.crypto.spec.GCMParameterSpec
import javax.crypto.spec.SecretKeySpec
import org.json.JSONObject
import org.junit.After
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertThrows
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class DubBridgeP2PKeyStoreInteropTest {
  private val module = DubBridgeP2PKeyStoreModule()

  @Before
  @After
  fun clearIdentity() {
    val keyStore = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
    if (keyStore.containsAlias(KEY_ALIAS)) {
      keyStore.deleteEntry(KEY_ALIAS)
    }
  }

  @Test
  fun bindingValidationRejectsExpiryAndIdentityDriftBeforeUnwrap() {
    val now = 2_000_000_000L
    val valid = nativeEnvelope(bindingJson(expiresAtUnix = now + 60))
    assertEquals(now + 60, validateEnvelopeBinding(valid, now).expiresAtUnix)

    assertThrows(IllegalArgumentException::class.java) {
      validateEnvelopeBinding(nativeEnvelope(bindingJson(expiresAtUnix = now)), now)
    }
    assertThrows(IllegalArgumentException::class.java) {
      validateEnvelopeBinding(
        nativeEnvelope(bindingJson(deviceKeyId = "another-device", expiresAtUnix = now + 60)),
        now,
      )
    }
    assertThrows(IllegalArgumentException::class.java) {
      validateEnvelopeBinding(
        nativeEnvelope(bindingJson(authorizationId = "", expiresAtUnix = now + 60)),
        now,
      )
    }
  }

  @Test
  fun opaqueAndroidKeystoreKeyCompletesHpkeInteropWithoutExport() {
    val keyPair = module.getOrCreateKeyPair()
    assertNull("K1 private key must remain non-exportable", keyPair.private.encoded)
    assertEquals("EC", keyPair.private.algorithm)

    val now = System.currentTimeMillis() / 1000
    val binding = bindingJson(expiresAtUnix = now + 300)
    val ck = ByteArray(32) { 7 }
    val sealed = sealForRecipient(keyPair.public as ECPublicKey, ck, binding)

    val recoveredBase64 = module.unwrapEnvelope(
      NativeHpkeEnvelope(
        profileVersion = PROFILE_VERSION,
        keyId = KEY_ALIAS,
        encapsulatedKeyBase64 = Base64.encodeToString(sealed.first, Base64.NO_WRAP),
        ciphertextBase64 = Base64.encodeToString(sealed.second, Base64.NO_WRAP),
        bindingJson = binding,
      ),
      now,
    )

    assertArrayEquals(ck, Base64.decode(recoveredBase64, Base64.DEFAULT))
    assertNull(module.getOrCreateKeyPair().private.encoded)
  }

  private fun nativeEnvelope(binding: String) = NativeHpkeEnvelope(
    profileVersion = PROFILE_VERSION,
    keyId = KEY_ALIAS,
    encapsulatedKeyBase64 = Base64.encodeToString(ByteArray(65) { 1 }, Base64.NO_WRAP),
    ciphertextBase64 = Base64.encodeToString(ByteArray(48) { 2 }, Base64.NO_WRAP),
    bindingJson = binding,
  )

  private fun bindingJson(
    deviceKeyId: String = KEY_ALIAS,
    authorizationId: String = "authorization-1",
    expiresAtUnix: Long,
  ): String = JSONObject()
    .put("profile_version", PROFILE_VERSION)
    .put("device_key_id", deviceKeyId)
    .put("invitation_id", "invitation-1")
    .put("viewer_id", "viewer-1")
    .put("asset_id", "asset-1")
    .put("publication_id", "publication-1")
    .put("lineage_id", "lineage-1")
    .put("authorization_id", authorizationId)
    .put("expires_at_unix", expiresAtUnix)
    .toString()

  private fun sealForRecipient(
    recipient: ECPublicKey,
    ck: ByteArray,
    bindingJson: String,
  ): Pair<ByteArray, ByteArray> {
    val generator = KeyPairGenerator.getInstance(KeyProperties.KEY_ALGORITHM_EC)
    generator.initialize(ECGenParameterSpec("secp256r1"))
    val ephemeral = generator.generateKeyPair()

    val agreement = KeyAgreement.getInstance("ECDH")
    agreement.init(ephemeral.private)
    agreement.doPhase(recipient, true)
    val dh = agreement.generateSecret()

    val encapsulated = encodeP256Point(ephemeral.public as ECPublicKey)
    val recipientPoint = encodeP256Point(recipient)
    val sharedSecret = extractAndExpandKem(dh, encapsulated + recipientPoint)
    dh.fill(0)

    val pskIdHash = labeledExtract(ByteArray(0), hpkeSuiteId(), "psk_id_hash", ByteArray(0))
    val infoHash = labeledExtract(
      ByteArray(0),
      hpkeSuiteId(),
      "info_hash",
      "dubbridge:p2p:k1:hpke-base:v1".toByteArray(StandardCharsets.US_ASCII),
    )
    val context = byteArrayOf(0) + pskIdHash + infoHash
    val secret = labeledExtract(sharedSecret, hpkeSuiteId(), "secret", ByteArray(0))
    sharedSecret.fill(0)
    val key = labeledExpand(secret, hpkeSuiteId(), "key", context, 32)
    val nonce = labeledExpand(secret, hpkeSuiteId(), "base_nonce", context, 12)
    secret.fill(0)

    return try {
      val cipher = Cipher.getInstance("AES/GCM/NoPadding")
      cipher.init(Cipher.ENCRYPT_MODE, SecretKeySpec(key, "AES"), GCMParameterSpec(128, nonce))
      cipher.updateAAD(bindingJson.toByteArray(StandardCharsets.UTF_8))
      encapsulated to cipher.doFinal(ck)
    } finally {
      key.fill(0)
      nonce.fill(0)
      context.fill(0)
    }
  }

  private fun extractAndExpandKem(dh: ByteArray, kemContext: ByteArray): ByteArray {
    val suite = byteArrayOf(
      'K'.code.toByte(), 'E'.code.toByte(), 'M'.code.toByte(),
      0x00, 0x10,
    )
    val eaePrk = labeledExtract(ByteArray(0), suite, "eae_prk", dh)
    return try {
      labeledExpand(eaePrk, suite, "shared_secret", kemContext, 32)
    } finally {
      eaePrk.fill(0)
    }
  }

  private fun labeledExtract(
    salt: ByteArray,
    suiteId: ByteArray,
    label: String,
    ikm: ByteArray,
  ): ByteArray {
    val labeledIkm = "HPKE-v1".toByteArray(StandardCharsets.US_ASCII) + suiteId +
      label.toByteArray(StandardCharsets.US_ASCII) + ikm
    val actualSalt = if (salt.isEmpty()) ByteArray(32) else salt
    return hmacSha256(actualSalt, labeledIkm)
  }

  private fun labeledExpand(
    prk: ByteArray,
    suiteId: ByteArray,
    label: String,
    info: ByteArray,
    length: Int,
  ): ByteArray {
    val lengthPrefix = byteArrayOf((length ushr 8).toByte(), length.toByte())
    val labeledInfo = lengthPrefix + "HPKE-v1".toByteArray(StandardCharsets.US_ASCII) +
      suiteId + label.toByteArray(StandardCharsets.US_ASCII) + info
    val output = ByteArray(length)
    var previous = ByteArray(0)
    var offset = 0
    var counter = 1
    while (offset < length) {
      val mac = Mac.getInstance("HmacSHA256")
      mac.init(SecretKeySpec(prk, "HmacSHA256"))
      mac.update(previous)
      mac.update(labeledInfo)
      mac.update(counter.toByte())
      val block = mac.doFinal()
      previous.fill(0)
      previous = block
      val count = minOf(block.size, length - offset)
      System.arraycopy(block, 0, output, offset, count)
      offset += count
      counter += 1
    }
    previous.fill(0)
    return output
  }

  private fun hmacSha256(key: ByteArray, data: ByteArray): ByteArray {
    val mac = Mac.getInstance("HmacSHA256")
    mac.init(SecretKeySpec(key, "HmacSHA256"))
    return mac.doFinal(data)
  }

  private fun hpkeSuiteId() = byteArrayOf(
    'H'.code.toByte(), 'P'.code.toByte(), 'K'.code.toByte(), 'E'.code.toByte(),
    0x00, 0x10,
    0x00, 0x01,
    0x00, 0x02,
  )

  private fun encodeP256Point(publicKey: ECPublicKey): ByteArray {
    val output = ByteArray(65)
    output[0] = 0x04
    copyCoordinate(publicKey.w.affineX, output, 1)
    copyCoordinate(publicKey.w.affineY, output, 33)
    return output
  }

  private fun copyCoordinate(value: BigInteger, target: ByteArray, offset: Int) {
    val raw = value.toByteArray()
    val first = if (raw.size == 33 && raw[0] == 0.toByte()) 1 else 0
    val length = raw.size - first
    require(length <= 32)
    System.arraycopy(raw, first, target, offset + (32 - length), length)
  }
}
