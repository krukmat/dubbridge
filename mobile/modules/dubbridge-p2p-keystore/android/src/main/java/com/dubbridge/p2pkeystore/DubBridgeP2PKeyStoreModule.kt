package com.dubbridge.p2pkeystore

import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition
import expo.modules.kotlin.records.Field
import expo.modules.kotlin.records.Record
import java.math.BigInteger
import java.nio.charset.StandardCharsets
import java.security.KeyFactory
import java.security.KeyPair
import java.security.KeyPairGenerator
import java.security.KeyStore
import java.security.PrivateKey
import java.security.interfaces.ECPublicKey
import java.security.spec.ECGenParameterSpec
import java.security.spec.ECPoint
import java.security.spec.ECPublicKeySpec
import javax.crypto.Cipher
import javax.crypto.KeyAgreement
import javax.crypto.Mac
import javax.crypto.spec.GCMParameterSpec
import javax.crypto.spec.SecretKeySpec

internal const val KEY_ALIAS = "dubbridge-p2p-k1-v1"
internal const val PROFILE_VERSION = "p2p-k1-hpke-v1"
private val HPKE_VERSION_LABEL = "HPKE-v1".toByteArray(StandardCharsets.US_ASCII)
private val KEM_SUITE_ID = byteArrayOf(
  'K'.code.toByte(), 'E'.code.toByte(), 'M'.code.toByte(),
  0x00, 0x10,
)
private val HPKE_SUITE_ID = byteArrayOf(
  'H'.code.toByte(), 'P'.code.toByte(), 'K'.code.toByte(), 'E'.code.toByte(),
  0x00, 0x10,
  0x00, 0x01,
  0x00, 0x02,
)
private val HPKE_INFO = "dubbridge:p2p:k1:hpke-base:v1".toByteArray(StandardCharsets.US_ASCII)

class HpkeEnvelopeRecord : Record {
  @Field
  val profileVersion: String = ""

  @Field
  val keyId: String = ""

  @Field
  val encapsulatedKeyBase64: String = ""

  @Field
  val ciphertextBase64: String = ""

  @Field
  val bindingJson: String = ""
}

class DubBridgeP2PKeyStoreModule : Module() {
  override fun definition() = ModuleDefinition {
    Name("DubBridgeP2PKeyStore")

    AsyncFunction("getOrCreateP256Identity") {
      val keyPair = getOrCreateKeyPair()
      mapOf(
        "keyId" to KEY_ALIAS,
        "publicKeySpkiBase64" to Base64.encodeToString(keyPair.public.encoded, Base64.NO_WRAP),
      )
    }

    AsyncFunction("unwrapHpkeBaseEnvelope") { envelope: HpkeEnvelopeRecord ->
      unwrapEnvelope(envelope.toNativeEnvelope())
    }
  }

  internal fun getOrCreateKeyPair(): KeyPair {
    val keyStore = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
    if (keyStore.containsAlias(KEY_ALIAS)) {
      return loadOpaqueKeyPair(keyStore)
    }

    val generator = KeyPairGenerator.getInstance(KeyProperties.KEY_ALGORITHM_EC, "AndroidKeyStore")
    val spec = KeyGenParameterSpec.Builder(KEY_ALIAS, KeyProperties.PURPOSE_AGREE_KEY)
      .setAlgorithmParameterSpec(ECGenParameterSpec("secp256r1"))
      .build()
    generator.initialize(spec)
    val generated = generator.generateKeyPair()
    requireOpaquePrivateKey(generated.private)
    require(generated.public is ECPublicKey) { "Android Keystore did not create a P-256 public key" }
    return generated
  }

  private fun loadOpaqueKeyPair(keyStore: KeyStore): KeyPair {
    val privateKey = keyStore.getKey(KEY_ALIAS, null) as? PrivateKey
      ?: error("K1 private key is unavailable")
    val publicKey = keyStore.getCertificate(KEY_ALIAS)?.publicKey as? ECPublicKey
      ?: error("K1 public key is unavailable")
    requireOpaquePrivateKey(privateKey)
    return KeyPair(publicKey, privateKey)
  }

  private fun requireOpaquePrivateKey(privateKey: PrivateKey) {
    check(privateKey.encoded == null) { "K1 private key must remain non-exportable" }
  }

  internal fun unwrapEnvelope(
    envelope: NativeHpkeEnvelope,
    nowUnix: Long = System.currentTimeMillis() / 1000,
  ): String {
    validateEnvelopeBinding(envelope, nowUnix)

    val encapsulated = decodeBase64(envelope.encapsulatedKeyBase64, "encapsulated key")
    val ciphertext = decodeBase64(envelope.ciphertextBase64, "ciphertext")
    require(encapsulated.size == 65 && encapsulated[0] == 0x04.toByte()) {
      "K1 encapsulated key is not an uncompressed P-256 point"
    }
    require(ciphertext.size == 48) { "K1 ciphertext must contain a 32-byte CK and GCM tag" }

    val keyPair = getOrCreateKeyPair()
    val recipientPublic = keyPair.public as ECPublicKey
    val ephemeralPublic = decodeP256Point(encapsulated, recipientPublic)
    val dh = deriveEcdh(keyPair.private, ephemeralPublic)
    try {
      val kemContext = encapsulated + encodeP256Point(recipientPublic)
      val sharedSecret = extractAndExpandKem(dh, kemContext)
      try {
        return decryptContentKey(sharedSecret, ciphertext, envelope.bindingJson)
      } finally {
        sharedSecret.fill(0)
      }
    } finally {
      dh.fill(0)
      ciphertext.fill(0)
    }
  }

  private fun deriveEcdh(privateKey: PrivateKey, peerPublic: ECPublicKey): ByteArray {
    requireOpaquePrivateKey(privateKey)
    val agreement = KeyAgreement.getInstance("ECDH", "AndroidKeyStore")
    agreement.init(privateKey)
    agreement.doPhase(peerPublic, true)
    return agreement.generateSecret()
  }

  private fun decodeP256Point(encoded: ByteArray, recipientPublic: ECPublicKey): ECPublicKey {
    require(encoded.size == 65 && encoded[0] == 0x04.toByte()) { "Invalid P-256 point" }
    val x = BigInteger(1, encoded.copyOfRange(1, 33))
    val y = BigInteger(1, encoded.copyOfRange(33, 65))
    val spec = ECPublicKeySpec(ECPoint(x, y), recipientPublic.params)
    return KeyFactory.getInstance("EC").generatePublic(spec) as ECPublicKey
  }

  private fun encodeP256Point(publicKey: ECPublicKey): ByteArray {
    val output = ByteArray(65)
    output[0] = 0x04
    copyUnsignedCoordinate(publicKey.w.affineX, output, 1)
    copyUnsignedCoordinate(publicKey.w.affineY, output, 33)
    return output
  }

  private fun copyUnsignedCoordinate(value: BigInteger, target: ByteArray, offset: Int) {
    val raw = value.toByteArray()
    val first = if (raw.size == 33 && raw[0] == 0.toByte()) 1 else 0
    val length = raw.size - first
    require(length <= 32) { "Invalid P-256 coordinate" }
    System.arraycopy(raw, first, target, offset + (32 - length), length)
  }

  private fun extractAndExpandKem(dh: ByteArray, kemContext: ByteArray): ByteArray {
    val eaePrk = labeledExtract(ByteArray(0), KEM_SUITE_ID, "eae_prk", dh)
    return try {
      labeledExpand(eaePrk, KEM_SUITE_ID, "shared_secret", kemContext, 32)
    } finally {
      eaePrk.fill(0)
    }
  }

  private fun decryptContentKey(
    sharedSecret: ByteArray,
    ciphertext: ByteArray,
    bindingJson: String,
  ): String {
    val pskIdHash = labeledExtract(ByteArray(0), HPKE_SUITE_ID, "psk_id_hash", ByteArray(0))
    val infoHash = labeledExtract(ByteArray(0), HPKE_SUITE_ID, "info_hash", HPKE_INFO)
    val context = byteArrayOf(0) + pskIdHash + infoHash
    pskIdHash.fill(0)
    infoHash.fill(0)

    val secret = labeledExtract(sharedSecret, HPKE_SUITE_ID, "secret", ByteArray(0))
    val key = labeledExpand(secret, HPKE_SUITE_ID, "key", context, 32)
    val nonce = labeledExpand(secret, HPKE_SUITE_ID, "base_nonce", context, 12)
    secret.fill(0)
    context.fill(0)

    try {
      val cipher = Cipher.getInstance("AES/GCM/NoPadding")
      cipher.init(Cipher.DECRYPT_MODE, SecretKeySpec(key, "AES"), GCMParameterSpec(128, nonce))
      cipher.updateAAD(bindingJson.toByteArray(StandardCharsets.UTF_8))
      val plaintext = cipher.doFinal(ciphertext)
      try {
        require(plaintext.size == 32) { "K1 envelope did not contain a 256-bit CK" }
        return Base64.encodeToString(plaintext, Base64.NO_WRAP)
      } finally {
        plaintext.fill(0)
      }
    } finally {
      key.fill(0)
      nonce.fill(0)
    }
  }

  private fun labeledExtract(
    salt: ByteArray,
    suiteId: ByteArray,
    label: String,
    ikm: ByteArray,
  ): ByteArray {
    val labeledIkm = HPKE_VERSION_LABEL + suiteId +
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
    require(length in 1..65535) { "Invalid HKDF output length" }
    val lengthPrefix = byteArrayOf((length ushr 8).toByte(), length.toByte())
    val labeledInfo = lengthPrefix + HPKE_VERSION_LABEL + suiteId +
      label.toByteArray(StandardCharsets.US_ASCII) + info
    return hkdfExpand(prk, labeledInfo, length)
  }

  private fun hmacSha256(key: ByteArray, data: ByteArray): ByteArray {
    val mac = Mac.getInstance("HmacSHA256")
    mac.init(SecretKeySpec(key, "HmacSHA256"))
    return mac.doFinal(data)
  }

  private fun hkdfExpand(prk: ByteArray, info: ByteArray, length: Int): ByteArray {
    require(length <= 255 * 32) { "HKDF output length is too large" }
    val output = ByteArray(length)
    var previous = ByteArray(0)
    var offset = 0
    var counter = 1

    while (offset < length) {
      val mac = Mac.getInstance("HmacSHA256")
      mac.init(SecretKeySpec(prk, "HmacSHA256"))
      mac.update(previous)
      mac.update(info)
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

  private fun decodeBase64(value: String, label: String): ByteArray {
    require(value.isNotBlank()) { "K1 $label is missing" }
    return try {
      Base64.decode(value, Base64.DEFAULT)
    } catch (error: IllegalArgumentException) {
      throw IllegalArgumentException("K1 $label is not valid base64", error)
    }
  }
}
